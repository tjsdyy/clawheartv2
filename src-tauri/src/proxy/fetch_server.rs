//! W12：反向代理服务器（监控模式 tier1 · 端点映射）
//!
//! 监听 :19112，提供 OpenAI / Anthropic 兼容路径。
//! Agent 配置 base_url 指向此处后：
//!   1. 提取 Authorization 头中的 sk-claw-xxx
//!   2. credential_router 反查 Profile + 真实 key
//!   3. 安全检查（W8 接 pipeline）
//!   4. reqwest 转发到真实 upstream
//!   5. 流式响应原样回传（SSE 边界保留）
//!   6. 命中安全策略时按协议格式伪造拦截响应（继承 v1 自适应优势）
//!
//! 仅在 `fetch_server` feature 启用时编译进二进制。

#![cfg(feature = "fetch_server")]

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use futures::TryStreamExt;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use tower_http::cors::{Any, CorsLayer};

use crate::proxy::credential_router::{
    build_upstream_url, route_for_authorization, ResolveError, RouteOutcome, UpstreamRouting,
};
use crate::proxy::security_check::{self, RequestLogEntry, TokenUsageEntry};
use crate::proxy::usage_extractor;
use crate::state::AppState;

// ──────────────────────────────────────────────────────────────────
// Public entry
// ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct FetchServerState {
    pub app: AppState,
    pub http: reqwest::Client,
}

pub struct FetchServer {
    pub bind: SocketAddr,
    pub state: FetchServerState,
}

impl FetchServer {
    pub fn new(app: AppState, port: u16) -> Self {
        let bind: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let http = reqwest::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            bind,
            state: FetchServerState { app, http },
        }
    }

    pub async fn run(self) -> Result<(), String> {
        let running_flag = self.state.app.fetch_server_running.clone();
        let port_flag = self.state.app.fetch_server_port.clone();
        let bind = self.bind;
        let port = bind.port();

        let app = router(self.state);
        let listener = match tokio::net::TcpListener::bind(bind).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!(error = %e, "fetch_server bind failed");
                running_flag.store(false, Ordering::Release);
                return Err(format!("bind {} failed: {}", bind, e));
            }
        };
        port_flag.store(port, Ordering::Release);
        running_flag.store(true, Ordering::Release);
        tracing::info!(addr = %bind, "fetch_server listening");

        let serve_result = axum::serve(listener, app).await;
        // 退出时（crash / shutdown）一律清标志
        running_flag.store(false, Ordering::Release);
        serve_result.map_err(|e| format!("axum serve error: {}", e))
    }
}

fn router(state: FetchServerState) -> Router {
    Router::new()
        // 健康检查
        .route("/healthz", get(healthz))
        // OpenAI Chat Completions
        .route("/v1/chat/completions", post(openai_chat))
        // Anthropic Messages
        .route("/v1/messages", post(anthropic_messages))
        // OpenAI Responses（Codex）
        .route("/v1/responses", post(openai_responses))
        // 通用透传（其他 path）
        .route("/v1/*path", post(passthrough_post))
        .route("/v1/*path", get(passthrough_get))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

// ──────────────────────────────────────────────────────────────────
// Handlers
// ──────────────────────────────────────────────────────────────────

async fn healthz() -> &'static str {
    "ok"
}

async fn openai_chat(
    State(state): State<FetchServerState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    forward_or_block(state, headers, body, "/chat/completions", "openai").await
}

async fn anthropic_messages(
    State(state): State<FetchServerState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    forward_or_block(state, headers, body, "/messages", "anthropic").await
}

async fn openai_responses(
    State(state): State<FetchServerState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    forward_or_block(state, headers, body, "/responses", "openai_responses").await
}

async fn passthrough_post(
    State(state): State<FetchServerState>,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    let routing = match route_for_authorization(&state.app, auth) {
        RouteOutcome::PassThrough => return text_error(StatusCode::BAD_REQUEST, "缺少有效 Authorization 头"),
        RouteOutcome::Reject(e) => return reject_response(&e, "unknown"),
        RouteOutcome::Rewrite(r) => r,
    };
    let target_path = format!("/{}", path);
    forward_raw(&state.http, &routing, &target_path, headers, body, "POST").await
}

async fn passthrough_get(
    State(state): State<FetchServerState>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    let routing = match route_for_authorization(&state.app, auth) {
        RouteOutcome::PassThrough => return text_error(StatusCode::BAD_REQUEST, "缺少有效 Authorization 头"),
        RouteOutcome::Reject(e) => return reject_response(&e, "unknown"),
        RouteOutcome::Rewrite(r) => r,
    };
    let target_path = format!("/{}", path);
    forward_raw(&state.http, &routing, &target_path, headers, Bytes::new(), "GET").await
}

// ──────────────────────────────────────────────────────────────────
// Core: 安全检查 → 路由 → 转发
// ──────────────────────────────────────────────────────────────────

async fn forward_or_block(
    state: FetchServerState,
    headers: HeaderMap,
    body: Value,
    upstream_path: &str,
    protocol: &str,
) -> Response {
    let started_at = std::time::Instant::now();
    let agent_id = headers
        .get("x-clawheart-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let model = body
        .get("model")
        .and_then(|v| v.as_str())
        .map(String::from);

    // 1. 反查虚拟 key（KillSwitch 由 security_check 处理）
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    let routing = match route_for_authorization(&state.app, auth) {
        RouteOutcome::PassThrough => {
            return adaptive_block_response(
                protocol,
                &body,
                "请求未携带 ClawHeart 虚拟 Key。请在 Agent 配置中将 API key 填入由 ClawHeart 发放的 sk-claw-* 凭据",
            );
        }
        RouteOutcome::Reject(e) => return reject_response(&e, protocol),
        RouteOutcome::Rewrite(r) => r,
    };

    // 2. 安全检查
    let text = security_check::collect_request_text(&body);
    let prompt_snippet = Some(text.chars().take(200).collect::<String>());
    let bytes_in = serde_json::to_vec(&body).map(|v| v.len() as i64).unwrap_or(0);

    let verdict = security_check::run_security_check(&state.app, &text);
    if let Some(meta) = verdict.meta() {
        security_check::record_intercept(
            &state.app,
            meta,
            agent_id.clone(),
            prompt_snippet.clone(),
        );
    }
    if verdict.is_block() {
        let reason = verdict
            .meta()
            .map(|m| m.reason.as_str())
            .unwrap_or("blocked");
        // 记 request_log（blocked=true）
        security_check::record_request_log(
            &state.app,
            RequestLogEntry {
                agent_id: agent_id.clone(),
                session_id: None,
                format: protocol.into(),
                provider: Some(routing.upstream_base_url.clone()),
                model: model.clone(),
                endpoint: upstream_path.into(),
                method: "POST".into(),
                status_code: 200, // 自适应响应仍是 200，body 含拦截提示
                blocked: true,
                bytes_in,
                bytes_out: 0,
                latency_ms: started_at.elapsed().as_millis() as i64,
            },
        );
        return adaptive_block_response(protocol, &body, reason);
    }

    // 3. 转发（保留 SSE 流式）
    let body_bytes = match serde_json::to_vec(&body) {
        Ok(b) => Bytes::from(b),
        Err(e) => {
            return text_error(StatusCode::BAD_REQUEST, &format!("body 序列化失败：{}", e))
        }
    };
    let bytes_in_actual = body_bytes.len() as i64;
    let forwarded = forward_raw_capturing(
        &state.http,
        &routing,
        upstream_path,
        headers,
        body_bytes,
        "POST",
    )
    .await;

    // 4. 记 request_log + token_usage（异步不阻塞响应）
    let latency_ms = started_at.elapsed().as_millis() as i64;
    let provider = Some(routing.upstream_base_url.clone());
    let format_str = protocol.to_string();
    let endpoint_str = upstream_path.to_string();

    match forwarded {
        ForwardResult::Streaming(resp) => {
            // 流式响应：暂记日志（usage 仅在流末尾才能拿到，留待 W14 真接 stream parser）
            security_check::record_request_log(
                &state.app,
                RequestLogEntry {
                    agent_id: agent_id.clone(),
                    session_id: None,
                    format: format_str,
                    provider,
                    model,
                    endpoint: endpoint_str,
                    method: "POST".into(),
                    status_code: 200,
                    blocked: false,
                    bytes_in: bytes_in_actual,
                    bytes_out: 0, // 流式：未知
                    latency_ms,
                },
            );
            resp
        }
        ForwardResult::Json { status, body: json_body, bytes_out, response } => {
            let log_id = security_check::record_request_log(
                &state.app,
                RequestLogEntry {
                    agent_id: agent_id.clone(),
                    session_id: None,
                    format: format_str.clone(),
                    provider: provider.clone(),
                    model: model.clone(),
                    endpoint: endpoint_str,
                    method: "POST".into(),
                    status_code: status as i32,
                    blocked: false,
                    bytes_in: bytes_in_actual,
                    bytes_out: bytes_out as i64,
                    latency_ms,
                },
            );

            // 解析 usage
            if let Some(json_val) = &json_body {
                let fmt = parse_llm_format(&format_str);
                if let Some(usage) = usage_extractor::extract(fmt, json_val) {
                    let provider_name = derive_provider_name(provider.as_deref().unwrap_or(""));
                    let model_name = model.unwrap_or_else(|| "unknown".into());
                    let cost = security_check::estimate_cost_usd(
                        &provider_name,
                        &model_name,
                        usage.input_tokens,
                        usage.output_tokens,
                    );
                    security_check::record_token_usage(
                        &state.app,
                        TokenUsageEntry {
                            request_log_id: log_id,
                            agent_id: agent_id.clone(),
                            provider: provider_name,
                            model: model_name,
                            input_tokens: usage.input_tokens,
                            output_tokens: usage.output_tokens,
                            cache_read: usage.cache_read_tokens,
                            cache_creation: usage.cache_creation_tokens,
                            cost_usd: cost,
                        },
                    );
                }
            }
            response
        }
        ForwardResult::Error(resp) => resp,
    }
}

/// 把 LlmFormat 字符串映射回 enum
fn parse_llm_format(s: &str) -> crate::proxy::formats::LlmFormat {
    use crate::proxy::formats::LlmFormat;
    match s {
        "openai" => LlmFormat::OpenAI,
        "anthropic" => LlmFormat::Claude,
        "openai_responses" => LlmFormat::OpenAIResponses,
        "gemini" => LlmFormat::Gemini,
        "ollama" => LlmFormat::Ollama,
        _ => LlmFormat::Unknown,
    }
}

/// 从 base_url 推断 provider 名（用于价格表查询）
fn derive_provider_name(base_url: &str) -> String {
    let lower = base_url.to_lowercase();
    if lower.contains("openrouter") { return "openrouter".into(); }
    if lower.contains("api.openai.com") { return "openai".into(); }
    if lower.contains("api.anthropic.com") { return "anthropic".into(); }
    if lower.contains(".openai.azure.com") { return "azure".into(); }
    if lower.contains("googleapis.com") { return "google".into(); }
    if lower.contains("deepbricks") { return "deepbricks".into(); }
    if lower.contains("localhost") || lower.contains("127.0.0.1") {
        return "litellm".into();
    }
    "custom".into()
}

/// forward_raw 的结果区分流式 / JSON / 错误三种，便于上层做日志/usage 解析
pub enum ForwardResult {
    /// 流式响应（SSE / chunked）—— 已经组装成 axum Response 流回 client
    Streaming(Response),
    /// 完整 JSON 响应 —— body 已解析且 axum Response 已构造
    Json {
        status: u16,
        body: Option<Value>,
        bytes_out: usize,
        response: Response,
    },
    /// 上游错误（reqwest 失败、build 错误等）
    Error(Response),
}

/// 与 forward_raw 行为一致，但返回结构化结果给上层做日志与 usage 解析
async fn forward_raw_capturing(
    http: &reqwest::Client,
    routing: &UpstreamRouting,
    path: &str,
    incoming_headers: HeaderMap,
    body: Bytes,
    method: &str,
) -> ForwardResult {
    let url = build_upstream_url(&routing.upstream_base_url, path);

    let mut upstream_headers = reqwest::header::HeaderMap::new();
    for (name, value) in incoming_headers.iter() {
        let lower = name.as_str().to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "host" | "content-length" | "authorization" | "connection" | "accept-encoding"
        ) {
            continue;
        }
        if let (Ok(n), Ok(v)) = (
            reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()),
            reqwest::header::HeaderValue::from_bytes(value.as_bytes()),
        ) {
            upstream_headers.insert(n, v);
        }
    }
    if let Ok(auth_v) = reqwest::header::HeaderValue::from_str(&routing.real_authorization) {
        upstream_headers.insert(reqwest::header::AUTHORIZATION, auth_v);
    }
    for (k, v) in &routing.extra_headers {
        if let (Ok(name), Ok(val)) = (
            reqwest::header::HeaderName::from_bytes(k.as_bytes()),
            reqwest::header::HeaderValue::from_str(v),
        ) {
            upstream_headers.insert(name, val);
        }
    }

    let method_enum = match method {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        _ => reqwest::Method::POST,
    };

    let req = http
        .request(method_enum, &url)
        .headers(upstream_headers)
        .body(body)
        .build();
    let req = match req {
        Ok(r) => r,
        Err(e) => {
            return ForwardResult::Error(text_error(
                StatusCode::BAD_GATEWAY,
                &format!("build upstream request: {}", e),
            ));
        }
    };

    let resp = match http.execute(req).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, url = %url, "upstream request failed");
            return ForwardResult::Error(text_error(
                StatusCode::BAD_GATEWAY,
                &format!("ClawHeart 上游请求失败：{}", e),
            ));
        }
    };

    // 判断响应是否流式：content-type 含 event-stream
    let is_stream = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("event-stream") || s.contains("chunked"))
        .unwrap_or(false);

    if is_stream {
        return ForwardResult::Streaming(relay_upstream(resp).await);
    }

    // 非流式：缓冲 body，解析 JSON
    let status = resp.status().as_u16();
    let headers = resp.headers().clone();
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return ForwardResult::Error(text_error(
                StatusCode::BAD_GATEWAY,
                &format!("读上游 body 失败：{}", e),
            ));
        }
    };
    let bytes_out = bytes.len();
    let body_json: Option<Value> = serde_json::from_slice(&bytes).ok();

    // 构造给 client 的响应（保留 status + headers + body）
    let mut builder = Response::builder().status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK));
    for (name, value) in headers.iter() {
        let n = name.as_str().to_ascii_lowercase();
        if matches!(
            n.as_str(),
            "connection" | "keep-alive" | "proxy-authenticate" | "proxy-authorization"
                | "te" | "trailers" | "transfer-encoding" | "upgrade" | "content-length"
        ) {
            continue;
        }
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            builder = builder.header(name, val);
        }
    }
    let response = builder
        .body(Body::from(bytes))
        .unwrap_or_else(|_| text_error(StatusCode::INTERNAL_SERVER_ERROR, "构建响应失败"));

    ForwardResult::Json {
        status,
        body: body_json,
        bytes_out,
        response,
    }
}

async fn forward_raw(
    http: &reqwest::Client,
    routing: &UpstreamRouting,
    path: &str,
    incoming_headers: HeaderMap,
    body: Bytes,
    method: &str,
) -> Response {
    let url = build_upstream_url(&routing.upstream_base_url, path);

    // 复制请求头（剔除 Host、Content-Length 等，由 reqwest 重新计算）
    let mut upstream_headers = reqwest::header::HeaderMap::new();
    for (name, value) in incoming_headers.iter() {
        let lower = name.as_str().to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "host" | "content-length" | "authorization" | "connection" | "accept-encoding"
        ) {
            continue;
        }
        if let (Ok(n), Ok(v)) = (
            reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()),
            reqwest::header::HeaderValue::from_bytes(value.as_bytes()),
        ) {
            upstream_headers.insert(n, v);
        }
    }
    // 替换 Authorization 为真实 key
    if let Ok(auth_v) = reqwest::header::HeaderValue::from_str(&routing.real_authorization) {
        upstream_headers.insert(reqwest::header::AUTHORIZATION, auth_v);
    }
    // 附加 profile 扩展头
    for (k, v) in &routing.extra_headers {
        if let (Ok(name), Ok(val)) = (
            reqwest::header::HeaderName::from_bytes(k.as_bytes()),
            reqwest::header::HeaderValue::from_str(v),
        ) {
            upstream_headers.insert(name, val);
        }
    }

    let method_enum = match method {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        _ => reqwest::Method::POST,
    };

    let req = http
        .request(method_enum, &url)
        .headers(upstream_headers)
        .body(body)
        .build();
    let req = match req {
        Ok(r) => r,
        Err(e) => return text_error(StatusCode::BAD_GATEWAY, &format!("build upstream request: {}", e)),
    };

    let resp = match http.execute(req).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, url = %url, "upstream request failed");
            return text_error(
                StatusCode::BAD_GATEWAY,
                &format!("ClawHeart 上游请求失败：{}", e),
            );
        }
    };

    relay_upstream(resp).await
}

/// 将 reqwest::Response 透传为 axum::Response（保留状态码、头与流式 body）
async fn relay_upstream(resp: reqwest::Response) -> Response {
    let status_u16 = resp.status().as_u16();
    let status = StatusCode::from_u16(status_u16).unwrap_or(StatusCode::OK);

    let mut builder = Response::builder().status(status);
    for (name, value) in resp.headers().iter() {
        let n = name.as_str().to_ascii_lowercase();
        // hop-by-hop 头不转发
        if matches!(
            n.as_str(),
            "connection"
                | "keep-alive"
                | "proxy-authenticate"
                | "proxy-authorization"
                | "te"
                | "trailers"
                | "transfer-encoding"
                | "upgrade"
                | "content-length" // 流式无固定长度
        ) {
            continue;
        }
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            builder = builder.header(name, val);
        }
    }

    // 流式 body：reqwest stream → axum Body
    let stream = resp
        .bytes_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
    let body = Body::from_stream(stream);
    builder.body(body).unwrap_or_else(|_| {
        text_error(StatusCode::INTERNAL_SERVER_ERROR, "构建响应失败")
    })
}

// ──────────────────────────────────────────────────────────────────
// 自适应拦截响应（继承 v1 的核心 UX 优势）
// 命中安全规则或 Kill Switch 时，按协议格式伪造响应，让拦截信息出现在 chat 气泡里
// ──────────────────────────────────────────────────────────────────

fn adaptive_block_response(protocol: &str, body: &Value, message: &str) -> Response {
    let is_stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
    let text = format!("⚠️ [ClawHeart 安全拦截]\n\n{}", message);

    if protocol == "anthropic" || body.get("system").is_some() {
        return anthropic_block(&text);
    }
    if is_stream {
        return openai_stream_block(&text);
    }
    openai_json_block(&text)
}

fn openai_json_block(text: &str) -> Response {
    let id = format!("chatcmpl-blocked-{}", now_unix_secs());
    let payload = serde_json::json!({
        "id": id,
        "object": "chat.completion",
        "created": now_unix_secs(),
        "model": "clawheart-block",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": text},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}
    });
    (StatusCode::OK, Json(payload)).into_response()
}

fn openai_stream_block(text: &str) -> Response {
    let id = format!("chatcmpl-blocked-{}", now_unix_secs());
    let chunk1 = serde_json::json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": now_unix_secs(),
        "model": "clawheart-block",
        "choices": [{
            "index": 0,
            "delta": {"role": "assistant", "content": text},
            "finish_reason": null
        }]
    });
    let chunk2 = serde_json::json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": now_unix_secs(),
        "model": "clawheart-block",
        "choices": [{
            "index": 0,
            "delta": {},
            "finish_reason": "stop"
        }]
    });

    let sse = format!(
        "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        chunk1, chunk2
    );
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream; charset=utf-8")
        .header("cache-control", "no-cache")
        .body(Body::from(sse))
        .unwrap_or_else(|_| text_error(StatusCode::OK, "blocked"))
}

fn anthropic_block(text: &str) -> Response {
    let id = format!("msg_blocked_{}", now_unix_secs());
    let sse = format!(
        "event: message_start\ndata: {}\n\n\
         event: content_block_start\ndata: {}\n\n\
         event: content_block_delta\ndata: {}\n\n\
         event: content_block_stop\ndata: {}\n\n\
         event: message_delta\ndata: {}\n\n\
         event: message_stop\ndata: {}\n\n",
        serde_json::json!({
            "type": "message_start",
            "message": {
                "id": id,
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": "clawheart-block",
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {"input_tokens": 0, "output_tokens": 1}
            }
        }),
        serde_json::json!({
            "type": "content_block_start",
            "index": 0,
            "content_block": {"type": "text", "text": ""}
        }),
        serde_json::json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {"type": "text_delta", "text": text}
        }),
        serde_json::json!({"type": "content_block_stop", "index": 0}),
        serde_json::json!({
            "type": "message_delta",
            "delta": {"stop_reason": "end_turn", "stop_sequence": null},
            "usage": {"output_tokens": 1}
        }),
        serde_json::json!({"type": "message_stop"})
    );
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream; charset=utf-8")
        .header("cache-control", "no-cache")
        .body(Body::from(sse))
        .unwrap_or_else(|_| text_error(StatusCode::OK, "blocked"))
}

fn reject_response(err: &ResolveError, protocol: &str) -> Response {
    let msg = match err {
        ResolveError::NotAVirtualKey => "请求未携带 ClawHeart 虚拟 Key".to_string(),
        ResolveError::UnknownVirtualKey => "虚拟 Key 不存在或已被删除".into(),
        ResolveError::StorageDisabled => "存储未启用，ClawHeart 无法路由请求".into(),
        ResolveError::KeychainFailure(s) => format!("凭据库访问失败：{}", s),
        ResolveError::DbFailure(s) => format!("数据库错误：{}", s),
        ResolveError::ProfileDisabled => "对应的 Provider Profile 已禁用".into(),
        ResolveError::NoCredential => "对应 Profile 未设置 API 凭据".into(),
    };
    adaptive_block_response(protocol, &Value::Null, &msg)
}

fn text_error(code: StatusCode, msg: &str) -> Response {
    Response::builder()
        .status(code)
        .header("content-type", "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "fatal").into_response())
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
