//! IPC: 从 Agent 配置文件反向导入渠道
//!
//! 参考 cc-switch 的 `import_*_providers_from_live` + 9router 的多 provider 结构。
//! 目前实现：OpenClaw（最常用、结构最规范）。
//! 后续可扩展：OpenCode / Codex / OpenEva 等。

use crate::agents::probes::{self, ChannelCandidate, IMPORTABLE_PLATFORMS};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use tauri::State;

/// 返回当前 ClawHeart 支持反向导入的平台列表（前端用来决定 UI）
#[tauri::command]
pub fn list_importable_platforms() -> AppResult<Vec<String>> {
    Ok(IMPORTABLE_PLATFORMS.iter().map(|s| s.to_string()).collect())
}

/// 扫描某 Agent 配置文件中的所有可导入渠道
#[tauri::command]
pub fn scan_importable_channels(
    _state: State<AppState>,
    agent_id: String,
) -> AppResult<Vec<ChannelCandidate>> {
    let platform = agent_id.split_once('/').map(|(p, _)| p).unwrap_or(&agent_id);

    let mut candidates: Vec<ChannelCandidate> = match platform {
        "openclaw" => probes::openclaw::extract_channels(&agent_id),
        "openeva" => probes::openeva::extract_channels(&agent_id),
        "opencode" => probes::opencode::extract_channels(&agent_id),
        "hermes" => probes::hermes::extract_channels(&agent_id),
        "claude" => probes::claude_code::extract_channels(&agent_id),
        "codex" => probes::codex_cli::extract_channels(&agent_id),
        "gemini" => probes::gemini_cli::extract_channels(&agent_id),
        // 其他平台暂不支持反向导入；前端会显示明确提示
        _ => return Ok(vec![]),
    };

    // 与现有 ClawHeart 渠道库比对，标记 already_exists
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(existing) = crate::storage::queries::providers::list(db) {
                let existing_urls: std::collections::HashSet<String> =
                    existing.iter().map(|p| p.base_url.clone()).collect();
                for c in &mut candidates {
                    c.already_exists = existing_urls.contains(&c.base_url);
                }
            }
        }
    }

    Ok(candidates)
}

/// 批量导入候选渠道
/// - 创建 provider profile（含 api_key 存 Keychain）
/// - 可选：分配给指定 Agent
/// 返回成功创建的 profile_id 列表
#[tauri::command]
pub fn import_channels_batch(
    _state: State<AppState>,
    candidates: Vec<ChannelCandidate>,
    assign_to_agent: Option<String>,
) -> AppResult<Vec<String>> {
    let mut created = Vec::new();

    #[cfg(feature = "storage")]
    {
        let Some(db) = &_state.db else {
            return Err(AppError::NotImplemented("storage feature disabled"));
        };

        for c in candidates {
            // 跳过 base_url 重复的（避免脏数据）
            if c.already_exists {
                tracing::info!(name = %c.name, base_url = %c.base_url, "skipped (already exists)");
                continue;
            }

            let id = format!(
                "imp-{:x}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_micros())
                    .unwrap_or(0)
            );
            let virtual_key = format!(
                "sk-claw-imp-{:x}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            );
            let credential_ref = format!("clawheart.provider.{}", id);

            // 拼一个易识别的名字：openai (from openclaw)
            let display_name = format!("{} (from {})", c.name, c.source_platform);

            // 插入 DB
            crate::storage::queries::providers::insert(
                db,
                &id,
                &display_name,
                &c.provider_kind,
                &c.protocol,
                &c.base_url,
                &credential_ref,
                c.default_model.as_deref(),
                None,
                &virtual_key,
                false, // is_default
                true,  // enabled
            )
            .map_err(|e| AppError::Other(format!("插入失败：{}", e)))?;

            // 若有 api_key 则存 Keychain
            if let Some(key) = &c.api_key {
                if !key.is_empty() {
                    let _ = crate::storage::keychain::store(&credential_ref, key);
                }
            }

            // 可选：分配给来源 Agent
            if let Some(target) = &assign_to_agent {
                let _ = crate::storage::queries::assignments::assign(db, target, &id);
            }

            tracing::info!(profile_id = %id, name = %display_name, "channel imported");
            created.push(id);
        }
        // 微小防抖（避免下游 query 拿到旧数据）
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let _ = assign_to_agent;
    Ok(created)
}
