//! Codex CLI 配置探测器（env 写入模式）
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
use crate::agents::DiscoveredAgent;
use std::path::PathBuf;

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
