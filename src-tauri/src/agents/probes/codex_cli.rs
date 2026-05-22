//! Codex CLI 配置探测器（env 写入模式）
//!
//! ## 反向导入
//! 同时支持解析 ~/.codex/config.toml 的 `[model_providers.xxx]` 多 section（cc-switch 与 9router 都用此结构）
//!
//! ## 原说明
//!
//!
//! Codex CLI 通过环境变量配置：
//!   OPENAI_API_BASE / OPENAI_BASE_URL
//!   OPENAI_API_KEY
//!
//! 通常用户在 shell rc 里 `export`：
//!   ~/.zshrc / ~/.bashrc / ~/.profile / ~/.bash_profile
//!
//! 接管策略（避免直接修改用户的 shell rc）：
//!   1. ClawHeart 写一个独立文件：~/.clawheart-v2/env-codex.sh
//!      → 内含两行 export 语句（指向 ClawHeart 反代 + 虚拟 key）
//!   2. 提示用户在自己的 shell rc 末尾加一行：
//!      [ -f ~/.clawheart-v2/env-codex.sh ] && source ~/.clawheart-v2/env-codex.sh
//!   3. 用户 source 后 OPENAI_API_BASE / OPENAI_API_KEY 即被覆盖
//!
//! 为什么不直接改 shell rc：
//!   - 用户的 shell rc 通常有大量自定义内容，自动修改风险高
//!   - 用户卸载 ClawHeart 后只要删那一行 source 即可
//!   - snapshot before/after 仅涉及 env-codex.sh，干净可控

use crate::agents::config_probe::*;
use crate::agents::probes::ChannelCandidate;
use crate::agents::DiscoveredAgent;
use std::path::PathBuf;

/// 从 ~/.codex/config.toml + ~/.codex/auth.json 提取所有 model_providers
pub fn extract_channels(agent_id: &str) -> Vec<ChannelCandidate> {
    let Some(home) = home_dir() else { return vec![]; };

    // 1. auth.json 拿 api_key + auth_mode（用于判断 OAuth）
    let auth_json: Option<serde_json::Value> =
        std::fs::read_to_string(home.join(".codex/auth.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());

    let api_key: Option<String> = auth_json.as_ref().and_then(|v| {
        v.get("OPENAI_API_KEY")
            .or_else(|| v.get("api_key"))
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    });

    // 识别 ChatGPT OAuth 模式（cc-switch / 9router 都用 auth_mode 字段标识）
    let is_oauth_account = auth_json
        .as_ref()
        .and_then(|v| v.get("auth_mode").and_then(|s| s.as_str()))
        .map(|m| m == "oauth" || m == "chatgpt")
        .unwrap_or(false)
        || auth_json
            .as_ref()
            .and_then(|v| v.get("tokens"))
            .is_some(); // 含 OAuth tokens 字段也算账号模式

    // 2. config.toml 简易解析 [model_providers.<name>] section
    let toml_path = home.join(".codex/config.toml");
    let Ok(content) = std::fs::read_to_string(&toml_path) else {
        return vec![];
    };

    let providers = parse_codex_toml_providers(&content);
    if providers.is_empty() {
        return vec![];
    }

    providers
        .into_iter()
        .map(|(name, fields)| {
            let base_url = fields
                .get("base_url")
                .or_else(|| fields.get("baseUrl"))
                .cloned()
                .unwrap_or_default();
            let wire_api = fields
                .get("wire_api")
                .cloned()
                .unwrap_or_else(|| "responses".into());
            let display_name = fields.get("name").cloned().unwrap_or_else(|| name.clone());
            let protocol = if wire_api.contains("anthropic") {
                "anthropic"
            } else if wire_api.contains("gemini") {
                "gemini"
            } else if wire_api.contains("responses") {
                "openai_responses"
            } else {
                "openai"
            };

            let mut warnings = Vec::new();
            if is_oauth_account && api_key.is_none() {
                warnings.push(
                    "Codex ChatGPT 账号登录模式：本机无原生 OpenAI API key，需用户手动补一个直连 key 才能用 ClawHeart 代理"
                        .into(),
                );
            } else if api_key.is_none() {
                warnings.push("~/.codex/auth.json 中无 OPENAI_API_KEY，导入后需手动配置".into());
            }
            if base_url.is_empty() {
                warnings.push("base_url 字段缺失".into());
            }

            ChannelCandidate {
                id: format!("codex:{}", name),
                name: display_name,
                source_agent_id: agent_id.to_string(),
                source_platform: "codex".to_string(),
                base_url,
                api_key: api_key.clone(),
                protocol: protocol.to_string(),
                default_model: None,
                provider_kind: "custom".to_string(),
                already_exists: false,
                warnings,
            }
        })
        .filter(|c| !c.base_url.is_empty())
        .collect()
}

/// 简易 TOML 解析：只提取 `[model_providers.<name>]` section + 其下 `key = "value"` 行
/// 返回 Vec<(provider_name, HashMap<field_key, value>)>
fn parse_codex_toml_providers(
    content: &str,
) -> Vec<(String, std::collections::HashMap<String, String>)> {
    use std::collections::HashMap;
    let mut out: Vec<(String, HashMap<String, String>)> = Vec::new();
    let mut current: Option<(String, HashMap<String, String>)> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // 检测 section [model_providers.<name>]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // 提交上一个
            if let Some((name, map)) = current.take() {
                if !name.is_empty() {
                    out.push((name, map));
                }
            }
            let section = &trimmed[1..trimmed.len() - 1];
            if let Some(rest) = section.strip_prefix("model_providers.") {
                current = Some((rest.trim().to_string(), HashMap::new()));
            } else {
                current = None;
            }
            continue;
        }
        // section 内的 key = value
        if let Some((_, map)) = current.as_mut() {
            if let Some(eq) = trimmed.find('=') {
                let key = trimmed[..eq].trim().to_string();
                let raw_val = trimmed[eq + 1..].trim();
                // 去除引号
                let value = raw_val
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .trim_start_matches('\'')
                    .trim_end_matches('\'')
                    .to_string();
                map.insert(key, value);
            }
        }
    }
    // 提交最后一个
    if let Some((name, map)) = current {
        if !name.is_empty() {
            out.push((name, map));
        }
    }
    out
}

pub struct CodexCliProbe;

/// ClawHeart 管理的 Codex env 文件
fn env_file_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".clawheart-v2/env-codex.sh"))
}

/// 用户可能的 shell rc 候选（仅用于 inspect_with_credential 扫描已有 export）
fn shell_rc_candidates() -> Vec<PathBuf> {
    let home = match home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    vec![
        home.join(".zshrc"),
        home.join(".bashrc"),
        home.join(".profile"),
        home.join(".bash_profile"),
        home.join(".config/fish/config.fish"),
    ]
}

fn read_text(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// 在 shell rc 文本中查找 `export OPENAI_API_BASE=...` 之类的行
/// 返回 (base_url, api_key) 都尽量找到
fn extract_codex_env_from_text(text: &str) -> (Option<String>, Option<String>) {
    let mut base = None;
    let mut key = None;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        // export FOO=bar 或 FOO=bar
        let body = line
            .strip_prefix("export ")
            .unwrap_or(line);
        let mut parts = body.splitn(2, '=');
        let name = parts.next().unwrap_or("").trim();
        let value_raw = parts.next().unwrap_or("").trim();
        if value_raw.is_empty() {
            continue;
        }
        // 去掉引号
        let value = value_raw
            .trim_start_matches('"')
            .trim_end_matches('"')
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .to_string();
        match name {
            "OPENAI_API_BASE" | "OPENAI_BASE_URL" => {
                if base.is_none() {
                    base = Some(value);
                }
            }
            "OPENAI_API_KEY" => {
                if key.is_none() {
                    key = Some(value);
                }
            }
            _ => {}
        }
    }
    (base, key)
}

/// 构造 env-codex.sh 内容
fn build_env_sh(base_url: &str, virtual_key: &str) -> String {
    format!(
        "# Managed by ClawHeart · Codex CLI 中转配置\n\
         # 在你的 shell rc（如 ~/.zshrc）末尾添加：\n\
         #   [ -f ~/.clawheart-v2/env-codex.sh ] && source ~/.clawheart-v2/env-codex.sh\n\
         #\n\
         export OPENAI_API_BASE=\"{}\"\n\
         export OPENAI_BASE_URL=\"{}\"\n\
         export OPENAI_API_KEY=\"{}\"\n",
        base_url, base_url, virtual_key
    )
}

impl ConfigProbe for CodexCliProbe {
    fn platform(&self) -> &'static str { "codex" }

    fn inspect(&self, agent: &DiscoveredAgent) -> ProbeResult {
        let env_path = env_file_path();
        let env_str = env_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let exists = env_path.as_ref().map(|p| p.exists()).unwrap_or(false);

        // 从 env-codex.sh 读 base_url（若 ClawHeart 已写过）
        let mut current_base_url = None;
        let mut current_key_present = false;
        if let Some(p) = &env_path {
            if exists {
                if let Some(text) = read_text(p) {
                    let (b, k) = extract_codex_env_from_text(&text);
                    current_base_url = b;
                    current_key_present = k
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);
                }
            }
        }

        let mut warnings = Vec::new();
        if !exists {
            warnings.push(
                "ClawHeart 尚未创建 env-codex.sh；首次应用时会自动生成".into(),
            );
        }
        warnings.push(
            "请确保你的 shell rc（~/.zshrc 等）已 source 此文件，否则 Codex 无法读到新环境变量".into(),
        );

        ProbeResult {
            agent_id: format!("{}/{}", agent.platform, agent.agent_name),
            agent_platform: "codex".into(),
            agent_name: agent.agent_name.clone(),
            current_base_url,
            current_key_present,
            config_source: ConfigSource::EnvVar {
                name: "OPENAI_API_BASE".into(),
                scope: format!("clawheart_managed:{}", env_str),
            },
            writable: env_path.is_some(),
            probe_available: true,
            warnings,
        }
    }

    fn plan_overwrite(
        &self,
        agent: &DiscoveredAgent,
        target: &OverwriteTarget<'_>,
    ) -> Option<ConfigPatch> {
        let env_path = env_file_path()?;
        let before = if env_path.exists() {
            read_text(&env_path).unwrap_or_default()
        } else {
            String::new()
        };
        let after = build_env_sh(target.base_url, target.virtual_key);
        let diff_lines = make_diff_lines(&before, &after);
        Some(ConfigPatch {
            agent_id: format!("{}/{}", agent.platform, agent.agent_name),
            agent_platform: "codex".into(),
            agent_name: agent.agent_name.clone(),
            source: ConfigSource::EnvVar {
                name: "OPENAI_API_BASE".into(),
                scope: format!(
                    "clawheart_managed:{}",
                    env_path.to_string_lossy()
                ),
            },
            before,
            after,
            diff_lines,
            // env 写入要求用户重启 shell 才生效；标记 Caution
            risk_level: PatchRisk::Caution,
        })
    }

    fn apply(&self, patch: &ConfigPatch, dry_run: bool) -> Result<AppliedPatch, String> {
        let real_path = env_file_path()
            .ok_or_else(|| "无法解析 HOME 目录".to_string())?;
        let write_path = if dry_run {
            dry_run_path("codex", "env-codex.sh")
        } else {
            real_path.clone()
        };
        ensure_parent(&write_path)?;
        std::fs::write(&write_path, &patch.after)
            .map_err(|e| format!("写入失败：{}", e))?;
        tracing::info!(
            platform = "codex",
            dry_run,
            path = %write_path.to_string_lossy(),
            "codex env file written"
        );
        Ok(AppliedPatch {
            config_path: write_path.to_string_lossy().to_string(),
            before_value: patch.before.clone(),
            after_value: patch.after.clone(),
            dry_run,
        })
    }

    fn rollback(
        &self,
        config_path: &str,
        before_value: &str,
        dry_run: bool,
    ) -> Result<(), String> {
        let path = if dry_run {
            dry_run_path("codex", "env-codex.sh")
        } else {
            PathBuf::from(config_path)
        };
        ensure_parent(&path)?;
        if before_value.is_empty() {
            // 原本就不存在 → 删除文件
            if path.exists() {
                std::fs::remove_file(&path).map_err(|e| format!("删除失败：{}", e))?;
            }
        } else {
            std::fs::write(&path, before_value).map_err(|e| format!("回滚失败：{}", e))?;
        }
        Ok(())
    }

    fn inspect_with_credential(
        &self,
        _agent: &DiscoveredAgent,
    ) -> Option<CredentialReadResult> {
        // 优先从 ClawHeart 管理的 env-codex.sh 读
        if let Some(env_path) = env_file_path() {
            if env_path.exists() {
                if let Some(text) = read_text(&env_path) {
                    let (base, key) = extract_codex_env_from_text(&text);
                    if let (Some(base_url), Some(api_key)) = (base, key) {
                        if !api_key.is_empty() && !api_key.starts_with("sk-claw-") {
                            return Some(CredentialReadResult {
                                base_url,
                                api_key,
                                source_path: env_path.to_string_lossy().to_string(),
                                source_label: "Codex · env-codex.sh".into(),
                            });
                        }
                    }
                }
            }
        }

        // 兜底：扫用户 shell rc，找用户已有的 OPENAI_API_BASE/KEY export
        for rc in shell_rc_candidates() {
            if !rc.exists() {
                continue;
            }
            let text = match read_text(&rc) {
                Some(t) => t,
                None => continue,
            };
            let (base, key) = extract_codex_env_from_text(&text);
            if let (Some(base_url), Some(api_key)) = (base, key) {
                if !api_key.is_empty() {
                    return Some(CredentialReadResult {
                        base_url,
                        api_key,
                        source_path: rc.to_string_lossy().to_string(),
                        source_label: format!(
                            "Codex · {}",
                            rc.file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("shell rc")
                        ),
                    });
                }
            }
        }
        None
    }
}
