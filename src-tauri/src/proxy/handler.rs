//! 请求/响应 handler — 实际跑安全管线的地方
//!
//! W5 真实接入 hudsucker 0.22 的 HttpHandler trait（async fn in trait, Rust 1.75+）。
//! W10 反查能力已就位（route_for_request_authorization）。
//!
//! 实际请求处理流程（proxy_real feature 启用时）：
//!   1. KillSwitch 检查 → 激活则立即拒绝
//!   2. 提取 Authorization 头中的虚拟 Key
//!   3. credential_router 反查 Profile + 真实 key
//!   4. 改写 Authorization + 重定向 URI 到真实 upstream
//!   5. 让 hudsucker 把改写后的请求转发出去

use std::sync::Arc;

use crate::state::AppState;

#[derive(Clone)]
pub struct ClawHeartHandler {
    pub kill_switch: Arc<crate::security::kill_switch::KillSwitch>,
    pub app: AppState,
}

impl ClawHeartHandler {
    pub fn new(
        kill_switch: Arc<crate::security::kill_switch::KillSwitch>,
        app: AppState,
    ) -> Self {
        Self { kill_switch, app }
    }

    /// W10 反查能力 —— 也可被 fetch_server 单独使用。
    pub fn route_for_request_authorization(
        &self,
        authorization_header: Option<&str>,
    ) -> Result<
        Option<crate::proxy::credential_router::UpstreamRouting>,
        crate::proxy::credential_router::ResolveError,
    > {
        use crate::proxy::credential_router::{route_for_authorization, RouteOutcome};

        if self.kill_switch.snapshot() {
            return Err(
                crate::proxy::credential_router::ResolveError::ProfileDisabled,
            );
        }

        match route_for_authorization(&self.app, authorization_header) {
            RouteOutcome::PassThrough => Ok(None),
            RouteOutcome::Rewrite(r) => Ok(Some(r)),
            RouteOutcome::Reject(e) => Err(e),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// hudsucker 0.22 HttpHandler 真实 impl（W5 spike）
// ─────────────────────────────────────────────────────────────────────
//
// 关键 API 要点（基于 hudsucker 0.22 docs）：
//   - trait HttpHandler 使用 async fn in trait（Rust 1.75+ 原生），不需要 async-trait
//   - `handle_request` 返回 `RequestOrResponse`；`req.into()` 即透传
//   - Body 类型 = `hudsucker::Body`；Request/Response 来自 `hudsucker::hyper`

#[cfg(feature = "proxy_real")]
mod real {
    use super::ClawHeartHandler;
    use crate::proxy::credential_router::{
        build_upstream_url, ResolveError, UpstreamRouting,
    };
    use crate::proxy::security_check::{
        self, is_llm_request_path, protocol_from_path, RequestLogEntry,
    };
    use http_body_util::{BodyExt, Full};
    use hudsucker::{
        hyper::{header, Method, Request, Response, Uri},
        Body, HttpContext, HttpHandler, RequestOrResponse, WebSocketContext, WebSocketHandler,
    };

    impl HttpHandler for ClawHeartHandler {
        async fn handle_request(
            &mut self,
            _ctx: &HttpContext,
            req: Request<Body>,
        ) -> RequestOrResponse {
            let started_at = std::time::Instant::now();

            // L7 KillSwitch
            if self.kill_switch.snapshot() {
                return blocked_response(503, "kill_switch", "ClawHeart Kill Switch active")
                    .into();
            }

            // 取关键元数据（提前 clone，避免 body 消费后失效）
            let method = req.method().clone();
            let path = req.uri().path().to_string();
            let auth_owned: Option<String> = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let content_type_is_json = req
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.contains("application/json"))
                .unwrap_or(false);
            let agent_id_header: Option<String> = req
                .headers()
                .get("x-clawheart-agent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            // 决定路由
            let routing_outcome = self.route_for_request_authorization(auth_owned.as_deref());

            // 是否需要 buffer body 做安全检查
            let should_check_body =
                method == Method::POST && is_llm_request_path(&path) && content_type_is_json;

            if !should_check_body {
                // 非 LLM 请求：仅做路由改写
                return match routing_outcome {
                    Ok(None) => req.into(),
                    Ok(Some(routing)) => {
                        let mut r = req;
                        if let Err(e) = rewrite_request(&mut r, &routing) {
                            return blocked_response(
                                502,
                                "clawheart_rewrite",
                                &format!("upstream rewrite failed: {}", e),
                            )
                            .into();
                        }
                        r.into()
                    }
                    Err(err) => {
                        let msg = describe_resolve_error(&err);
                        blocked_response(403, "clawheart_route", &msg).into()
                    }
                };
            }

            // LLM 请求：buffer body
            let (mut parts, body) = req.into_parts();
            let body_bytes = match body.collect().await {
                Ok(c) => c.to_bytes(),
                Err(e) => {
                    return blocked_response(
                        502,
                        "clawheart_body_read",
                        &format!("read request body failed: {}", e),
                    )
                    .into();
                }
            };
            let bytes_in = body_bytes.len() as i64;

            // 解析 JSON 做安全检查（解析失败时不阻塞，继续透传）
            let json_value: Option<serde_json::Value> =
                serde_json::from_slice(&body_bytes).ok();
            let prompt_snippet = json_value
                .as_ref()
                .map(|v| security_check::collect_request_text(v));
            let snippet_for_log = prompt_snippet
                .as_ref()
                .map(|t| t.chars().take(200).collect::<String>());

            if let Some(text) = &prompt_snippet {
                let verdict = security_check::run_security_check(&self.app, text);
                if let Some(meta) = verdict.meta() {
                    security_check::record_intercept(
                        &self.app,
                        meta,
                        agent_id_header.clone(),
                        snippet_for_log.clone(),
                    );
                }
                if verdict.is_block() {
                    let reason = verdict
                        .meta()
                        .map(|m| m.reason.clone())
                        .unwrap_or_else(|| "blocked".into());
                    let proto = protocol_from_path(&path).to_string();
                    let routing_provider = match &routing_outcome {
                        Ok(Some(r)) => Some(r.upstream_base_url.clone()),
                        _ => None,
                    };
                    let model_name = json_value
                        .as_ref()
                        .and_then(|v| v.get("model"))
                        .and_then(|m| m.as_str())
                        .map(String::from);
                    security_check::record_request_log(
                        &self.app,
                        RequestLogEntry {
                            agent_id: agent_id_header.clone(),
                            session_id: None,
                            format: proto,
                            provider: routing_provider,
                            model: model_name,
                            endpoint: path.clone(),
                            method: "POST".into(),
                            status_code: 403,
                            blocked: true,
                            bytes_in,
                            bytes_out: 0,
                            latency_ms: started_at.elapsed().as_millis() as i64,
                        },
                    );
                    return blocked_response(403, "clawheart_security", &reason).into();
                }
            }

            // 应用路由改写到 parts
            match routing_outcome {
                Ok(None) => {}
                Ok(Some(routing)) => {
                    if let Err(e) = rewrite_parts(&mut parts, &routing) {
                        return blocked_response(
                            502,
                            "clawheart_rewrite",
                            &format!("upstream rewrite failed: {}", e),
                        )
                        .into();
                    }
                }
                Err(err) => {
                    let msg = describe_resolve_error(&err);
                    return blocked_response(403, "clawheart_route", &msg).into();
                }
            }

            // 重建 Request（保留改写后的 parts + 原始 body bytes）
            // hudsucker::Body 没有 From<Bytes>，但有 From<Full<Bytes>>
            let new_body = Body::from(Full::new(body_bytes));
            let new_req = Request::from_parts(parts, new_body);
            new_req.into()
        }

        async fn handle_response(
            &mut self,
            _ctx: &HttpContext,
            res: Response<Body>,
        ) -> Response<Body> {
            // L2 流式扫描状态机（W11+ 接 pipeline）；当前透传
            res
        }
    }

    // hudsucker 0.22 要求 handler 同时实现 WebSocketHandler。
    // 所有 WebSocket 帧透传 — 用默认 handle_websocket 即可，只覆盖 handle_message。
    impl WebSocketHandler for ClawHeartHandler {
        async fn handle_message(
            &mut self,
            _ctx: &WebSocketContext,
            message: hudsucker::tokio_tungstenite::tungstenite::Message,
        ) -> Option<hudsucker::tokio_tungstenite::tungstenite::Message> {
            // W11+ 接 MCP-over-WebSocket 协议扫描
            Some(message)
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // 改写请求：替换 Authorization + 改写 URI 指向真实 upstream
    // ──────────────────────────────────────────────────────────────────

    fn rewrite_request(
        req: &mut Request<Body>,
        routing: &UpstreamRouting,
    ) -> Result<(), String> {
        // Authorization
        let auth_v = header::HeaderValue::from_str(&routing.real_authorization)
            .map_err(|e| format!("auth header invalid: {}", e))?;
        req.headers_mut().insert(header::AUTHORIZATION, auth_v);

        // 扩展请求头（Profile.headers_json）
        for (k, v) in &routing.extra_headers {
            let name = header::HeaderName::from_bytes(k.as_bytes())
                .map_err(|e| format!("header name invalid: {}", e))?;
            let val = header::HeaderValue::from_str(v)
                .map_err(|e| format!("header val invalid: {}", e))?;
            req.headers_mut().insert(name, val);
        }

        // URI 改写
        let path_and_query = req
            .uri()
            .path_and_query()
            .map(|p| p.as_str().to_string())
            .unwrap_or_else(|| "/".into());
        let upstream_url = build_upstream_url(&routing.upstream_base_url, &path_and_query);
        let new_uri = upstream_url
            .parse::<Uri>()
            .map_err(|e| format!("uri parse error: {}", e))?;

        // Host 头同步
        if let Some(host) = new_uri.host() {
            let host_v = header::HeaderValue::from_str(host)
                .map_err(|e| format!("host header invalid: {}", e))?;
            req.headers_mut().insert(header::HOST, host_v);
        }
        *req.uri_mut() = new_uri;
        Ok(())
    }

    /// 改写 Request 的各 part（headers + uri），用于 from_parts 重建场景
    fn rewrite_parts(
        parts: &mut hudsucker::hyper::http::request::Parts,
        routing: &UpstreamRouting,
    ) -> Result<(), String> {
        let auth_v = header::HeaderValue::from_str(&routing.real_authorization)
            .map_err(|e| format!("auth header invalid: {}", e))?;
        parts.headers.insert(header::AUTHORIZATION, auth_v);

        for (k, v) in &routing.extra_headers {
            let name = header::HeaderName::from_bytes(k.as_bytes())
                .map_err(|e| format!("header name invalid: {}", e))?;
            let val = header::HeaderValue::from_str(v)
                .map_err(|e| format!("header val invalid: {}", e))?;
            parts.headers.insert(name, val);
        }

        let path_and_query = parts
            .uri
            .path_and_query()
            .map(|p| p.as_str().to_string())
            .unwrap_or_else(|| "/".into());
        let upstream_url = build_upstream_url(&routing.upstream_base_url, &path_and_query);
        let new_uri = upstream_url
            .parse::<Uri>()
            .map_err(|e| format!("uri parse error: {}", e))?;
        if let Some(host) = new_uri.host() {
            let host_v = header::HeaderValue::from_str(host)
                .map_err(|e| format!("host header invalid: {}", e))?;
            parts.headers.insert(header::HOST, host_v);
        }
        parts.uri = new_uri;
        Ok(())
    }

    fn blocked_response(status: u16, kind: &str, message: &str) -> Response<Body> {
        let body = serde_json::json!({
            "error": {
                "type": kind,
                "message": message,
            }
        });
        let body_text = body.to_string();
        Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(Body::from(body_text))
            .unwrap_or_else(|_| Response::new(Body::empty()))
    }

    fn describe_resolve_error(err: &ResolveError) -> String {
        match err {
            ResolveError::NotAVirtualKey => "请求未携带 ClawHeart 虚拟 Key".into(),
            ResolveError::UnknownVirtualKey => "虚拟 Key 不存在或已被删除".into(),
            ResolveError::StorageDisabled => "存储未启用，ClawHeart 无法路由请求".into(),
            ResolveError::KeychainFailure(s) => format!("凭据库访问失败：{}", s),
            ResolveError::DbFailure(s) => format!("数据库错误：{}", s),
            ResolveError::ProfileDisabled => "对应的 Provider Profile 已禁用".into(),
            ResolveError::NoCredential => "对应 Profile 未设置 API 凭据".into(),
        }
    }
}

#[cfg(not(feature = "proxy_real"))]
impl ClawHeartHandler {
    pub fn note(&self) -> &'static str {
        "alpha stub — enable feature `proxy_real` to use hudsucker"
    }
}
