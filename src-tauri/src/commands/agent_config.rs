//! IPC: Agent 配置一键覆盖（W7）
//!
//! 设计：[`docs/proposals/agent-config-autoapply.md`]
//!
//! 命令清单：
//!   scan_agent_configs           — 探测所有 Agent 当前配置
//!   plan_overwrite               — 计算覆盖 patch（含 diff）
//!   apply_overwrite              — 写入 + 创建 snapshot（默认 dry_run）
//!   list_apply_batches           — 列出历史批次摘要
//!   list_batch_snapshots         — 列出某批次的快照
//!   rollback_batch               — 回滚整个批次
//!   rollback_snapshot            — 回滚单个 snapshot

use crate::agents::config_probe::{
    self, ConfigPatch, ConfigSource, OverwriteTarget, PatchRisk, ProbeResult,
};
use crate::agents::scanner::Scanner;
use crate::agents::DiscoveredAgent;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize)]
pub struct ApplyOutcomeDto {
    pub agent_id: String,
    pub agent_platform: String,
    pub agent_name: String,
    pub success: bool,
    pub snapshot_id: Option<String>,
    pub config_path: String,
    pub message: String,
    pub dry_run: bool,
}

#[derive(Serialize)]
pub struct ApplyBatchResult {
    pub batch_id: String,
    pub dry_run: bool,
    pub outcomes: Vec<ApplyOutcomeDto>,
    pub success_count: u32,
    pub failure_count: u32,
}

#[derive(Serialize, Clone)]
pub struct BatchSummaryDto {
    pub batch_id: String,
    pub profile_id: Option<String>,
    pub agent_count: u32,
    pub applied_at: String,
    pub fully_rolled_back: bool,
}

#[derive(Serialize, Clone)]
pub struct SnapshotDto {
    pub id: String,
    pub batch_id: String,
    pub agent_id: String,
    pub agent_platform: String,
    pub config_path: String,
    pub applied_at: String,
    pub rolled_back_at: Option<String>,
}

#[derive(Serialize)]
pub struct RollbackResult {
    pub batch_id: Option<String>,
    pub snapshots_total: u32,
    pub snapshots_restored: u32,
    pub failures: Vec<String>,
}

#[derive(Deserialize)]
pub struct PlanInput {
    pub profile_id: String,
    pub agent_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct ApplyInput {
    pub profile_id: String,
    pub patches: Vec<ConfigPatch>,
    pub dry_run: Option<bool>,
}

#[derive(Serialize, Clone)]
pub struct ApplyRealStatus {
    pub enabled: bool,
    pub forced_dry_run: bool, // true 表示后端强制走 dry-run，不论前端如何要求
    pub setting_key: String,  // "apply_real_enabled"
}

// ──────────────────────────────────────────────────────────────────
// helpers
// ──────────────────────────────────────────────────────────────────

fn now_uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let nano = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:013x}-{:08x}", ms, nano)
}

fn config_kind_label(src: &ConfigSource) -> &'static str {
    match src {
        ConfigSource::JsonFile { .. } => "json_file",
        ConfigSource::TomlFile { .. } => "toml_file",
        ConfigSource::EnvVar { .. } => "env_var",
        ConfigSource::VsCodeWorkspace { .. } => "vsx_setting",
        ConfigSource::Unknown => "unknown",
    }
}

fn config_path_str(src: &ConfigSource) -> String {
    match src {
        ConfigSource::JsonFile { path, .. } => path.clone(),
        ConfigSource::TomlFile { path, .. } => path.clone(),
        ConfigSource::EnvVar { name, .. } => format!("env:{}", name),
        ConfigSource::VsCodeWorkspace { path, .. } => path.clone(),
        ConfigSource::Unknown => "".into(),
    }
}

fn discover_agents() -> Vec<DiscoveredAgent> {
    Scanner::with_default_platforms().scan_once()
}

/// 读取"实际写入"开关。默认 false（强制 dry-run，安全优先）
fn read_apply_real_enabled(state: &State<AppState>) -> bool {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            if let Ok(Some(v)) =
                crate::storage::queries::settings::get(db, "apply_real_enabled")
            {
                return v == "true" || v == "1";
            }
        }
    }
    let _ = state;
    false
}

fn write_apply_real_enabled(state: &State<AppState>, enabled: bool) {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let _ = crate::storage::queries::settings::set(
                db,
                "apply_real_enabled",
                if enabled { "true" } else { "false" },
            );
        }
    }
    let _ = (state, enabled);
}

// ──────────────────────────────────────────────────────────────────
// IPC: scan
// ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn scan_agent_configs() -> AppResult<Vec<ProbeResult>> {
    let agents = discover_agents();
    let mut results = Vec::new();
    for agent in &agents {
        match config_probe::probe_for(&agent.platform) {
            Some(probe) => results.push(probe.inspect(agent)),
            None => results.push(ProbeResult {
                agent_id: format!("{}/{}", agent.platform, agent.agent_name),
                agent_platform: agent.platform.clone(),
                agent_name: agent.agent_name.clone(),
                current_base_url: None,
                current_key_present: false,
                config_source: ConfigSource::Unknown,
                writable: false,
                probe_available: false,
                warnings: vec![format!("平台 {} 暂未支持自动接管", agent.platform)],
            }),
        }
    }
    Ok(results)
}

// ──────────────────────────────────────────────────────────────────
// IPC: plan
// ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn plan_overwrite(
    state: State<AppState>,
    input: PlanInput,
) -> AppResult<Vec<ConfigPatch>> {
    let (base_url, virtual_key, protocol) = resolve_target(&state, &input.profile_id)?;
    let agents = discover_agents();
    let target = OverwriteTarget {
        base_url: &base_url,
        virtual_key: &virtual_key,
        protocol: &protocol,
        profile_id: &input.profile_id,
    };
    let mut patches = Vec::new();
    for agent in &agents {
        let agent_id = format!("{}/{}", agent.platform, agent.agent_name);
        if !input.agent_ids.is_empty() && !input.agent_ids.contains(&agent_id) {
            continue;
        }
        if let Some(probe) = config_probe::probe_for(&agent.platform) {
            if let Some(patch) = probe.plan_overwrite(agent, &target) {
                patches.push(patch);
            }
        }
    }
    Ok(patches)
}

// ──────────────────────────────────────────────────────────────────
// IPC: apply
// ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn apply_overwrite(
    state: State<AppState>,
    input: ApplyInput,
) -> AppResult<ApplyBatchResult> {
    let requested_dry_run = input.dry_run.unwrap_or(true);
    let real_enabled = read_apply_real_enabled(&state);

    // 安全保险：若用户未在「设置 → 安全」中明确开启"实际写入"，
    // 不论前端如何要求，一律强制 dry-run。
    let dry_run = if !real_enabled { true } else { requested_dry_run };
    if !real_enabled && !requested_dry_run {
        tracing::warn!(
            "apply_overwrite: 前端请求实际写入但全局开关未开启，强制降级为 dry-run"
        );
    }

    let batch_id = now_uuid();
    let mut outcomes = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;

    for patch in input.patches {
        let probe = match config_probe::probe_for(&patch.agent_platform) {
            Some(p) => p,
            None => {
                failure_count += 1;
                outcomes.push(ApplyOutcomeDto {
                    agent_id: patch.agent_id.clone(),
                    agent_platform: patch.agent_platform.clone(),
                    agent_name: patch.agent_name.clone(),
                    success: false,
                    snapshot_id: None,
                    config_path: config_path_str(&patch.source),
                    message: format!("平台 {} 暂未支持", patch.agent_platform),
                    dry_run,
                });
                continue;
            }
        };

        match probe.apply(&patch, dry_run) {
            Ok(applied) => {
                let snapshot_id = now_uuid();
                #[cfg(feature = "storage")]
                {
                    if let Some(db) = &state.db {
                        let _ = crate::storage::queries::agent_config::insert_snapshot(
                            db,
                            &snapshot_id,
                            &batch_id,
                            &patch.agent_id,
                            &patch.agent_platform,
                            &applied.config_path,
                            config_kind_label(&patch.source),
                            &applied.before_value,
                            &applied.after_value,
                            Some(&input.profile_id),
                        );
                    }
                }
                let _ = &state;
                success_count += 1;
                outcomes.push(ApplyOutcomeDto {
                    agent_id: patch.agent_id.clone(),
                    agent_platform: patch.agent_platform.clone(),
                    agent_name: patch.agent_name.clone(),
                    success: true,
                    snapshot_id: Some(snapshot_id),
                    config_path: applied.config_path,
                    message: if dry_run {
                        "已写入 dry-run 沙箱".into()
                    } else {
                        "已写入目标配置".into()
                    },
                    dry_run: applied.dry_run,
                });
            }
            Err(e) => {
                failure_count += 1;
                outcomes.push(ApplyOutcomeDto {
                    agent_id: patch.agent_id.clone(),
                    agent_platform: patch.agent_platform.clone(),
                    agent_name: patch.agent_name.clone(),
                    success: false,
                    snapshot_id: None,
                    config_path: config_path_str(&patch.source),
                    message: e,
                    dry_run,
                });
            }
        }
    }

    tracing::info!(
        batch_id = %batch_id,
        dry_run,
        success_count,
        failure_count,
        "agent config overwrite batch applied"
    );

    Ok(ApplyBatchResult {
        batch_id,
        dry_run,
        outcomes,
        success_count,
        failure_count,
    })
}

// ──────────────────────────────────────────────────────────────────
// IPC: list batches / snapshots
// ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_apply_batches(state: State<AppState>) -> AppResult<Vec<BatchSummaryDto>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let rows = crate::storage::queries::agent_config::list_batches(db)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(rows
                .into_iter()
                .map(|r| BatchSummaryDto {
                    batch_id: r.batch_id,
                    profile_id: r.profile_id,
                    agent_count: r.agent_count,
                    applied_at: r.applied_at,
                    fully_rolled_back: r.fully_rolled_back,
                })
                .collect());
        }
    }
    let _ = state;
    Ok(vec![])
}

#[tauri::command]
pub fn list_batch_snapshots(
    state: State<AppState>,
    batch_id: String,
) -> AppResult<Vec<SnapshotDto>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let rows = crate::storage::queries::agent_config::list_snapshots_by_batch(
                db, &batch_id,
            )
            .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(rows
                .into_iter()
                .map(|r| SnapshotDto {
                    id: r.id,
                    batch_id: r.batch_id,
                    agent_id: r.agent_id,
                    agent_platform: r.agent_platform,
                    config_path: r.config_path,
                    applied_at: r.applied_at,
                    rolled_back_at: r.rolled_back_at,
                })
                .collect());
        }
    }
    let _ = (state, batch_id);
    Ok(vec![])
}

// ──────────────────────────────────────────────────────────────────
// IPC: apply-real 全局开关
// ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_apply_real_status(state: State<AppState>) -> AppResult<ApplyRealStatus> {
    let enabled = read_apply_real_enabled(&state);
    Ok(ApplyRealStatus {
        enabled,
        forced_dry_run: !enabled,
        setting_key: "apply_real_enabled".into(),
    })
}

#[tauri::command]
pub fn set_apply_real_enabled(
    state: State<AppState>,
    enabled: bool,
    acknowledged: bool,
) -> AppResult<ApplyRealStatus> {
    if enabled && !acknowledged {
        return Err(AppError::Other(
            "开启实际写入需要 acknowledged=true（请先在 UI 二次确认）".into(),
        ));
    }
    write_apply_real_enabled(&state, enabled);
    tracing::warn!(
        enabled,
        "apply_real_enabled toggled — 影响后续所有 apply_overwrite 操作"
    );
    Ok(ApplyRealStatus {
        enabled,
        forced_dry_run: !enabled,
        setting_key: "apply_real_enabled".into(),
    })
}

// ──────────────────────────────────────────────────────────────────
// IPC: rollback
// ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn rollback_batch(
    state: State<AppState>,
    batch_id: String,
    dry_run: Option<bool>,
) -> AppResult<RollbackResult> {
    let requested_dry_run = dry_run.unwrap_or(true);
    let real_enabled = read_apply_real_enabled(&state);
    let dry_run = if !real_enabled { true } else { requested_dry_run };
    let mut restored = 0u32;
    let mut total = 0u32;
    let mut failures: Vec<String> = Vec::new();

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let rows = crate::storage::queries::agent_config::list_snapshots_by_batch(
                db, &batch_id,
            )
            .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            total = rows.len() as u32;
            for row in rows {
                if row.rolled_back_at.is_some() {
                    continue;
                }
                let probe = match config_probe::probe_for(&row.agent_platform) {
                    Some(p) => p,
                    None => {
                        failures.push(format!("{}：未支持平台", row.agent_platform));
                        continue;
                    }
                };
                match probe.rollback(&row.config_path, &row.before_value, dry_run) {
                    Ok(()) => {
                        let _ = crate::storage::queries::agent_config::mark_rolled_back(db, &row.id);
                        restored += 1;
                    }
                    Err(e) => failures.push(format!("{}：{}", row.agent_id, e)),
                }
            }
        }
    }

    let _ = state;
    tracing::info!(
        batch_id = %batch_id,
        dry_run,
        total,
        restored,
        failures = failures.len(),
        "batch rollback completed"
    );
    Ok(RollbackResult {
        batch_id: Some(batch_id),
        snapshots_total: total,
        snapshots_restored: restored,
        failures,
    })
}

#[tauri::command]
pub fn rollback_snapshot(
    state: State<AppState>,
    snapshot_id: String,
    dry_run: Option<bool>,
) -> AppResult<RollbackResult> {
    let requested_dry_run = dry_run.unwrap_or(true);
    let real_enabled = read_apply_real_enabled(&state);
    let dry_run = if !real_enabled { true } else { requested_dry_run };

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let row = crate::storage::queries::agent_config::get_snapshot(db, &snapshot_id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?
                .ok_or_else(|| AppError::Other("Snapshot 不存在".into()))?;
            if row.rolled_back_at.is_some() {
                return Ok(RollbackResult {
                    batch_id: Some(row.batch_id),
                    snapshots_total: 1,
                    snapshots_restored: 0,
                    failures: vec!["该快照已回滚".into()],
                });
            }
            let probe = config_probe::probe_for(&row.agent_platform)
                .ok_or_else(|| AppError::Other("未支持平台".into()))?;
            probe
                .rollback(&row.config_path, &row.before_value, dry_run)
                .map_err(AppError::Other)?;
            let _ = crate::storage::queries::agent_config::mark_rolled_back(db, &row.id);
            tracing::info!(snapshot_id = %row.id, dry_run, "snapshot rolled back");
            return Ok(RollbackResult {
                batch_id: Some(row.batch_id),
                snapshots_total: 1,
                snapshots_restored: 1,
                failures: vec![],
            });
        }
    }
    let _ = (state, snapshot_id);
    Err(AppError::NotImplemented("storage feature disabled"))
}

// ──────────────────────────────────────────────────────────────────
// internal: resolve profile to (base_url, virtual_key, protocol)
// ──────────────────────────────────────────────────────────────────

fn resolve_target(
    state: &State<AppState>,
    profile_id: &str,
) -> Result<(String, String, String), AppError> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let row = crate::storage::queries::providers::get(db, profile_id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?
                .ok_or_else(|| AppError::Other("Profile 不存在".into()))?;
            // Agent 内不直接放 base_url 真实地址，而是 ClawHeart 本地代理端点 + 虚拟 key
            // base_url 取当前 access_mode 的 fetch_url_template
            let port = crate::storage::queries::settings::get(db, "reverse_proxy_port")
                .ok()
                .flatten()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(19112);
            let local_base = format!("http://127.0.0.1:{}/v1", port);
            return Ok((local_base, row.virtual_key, row.protocol));
        }
    }
    let _ = (state, profile_id);
    Err(AppError::NotImplemented("storage feature disabled"))
}

#[allow(dead_code)]
fn risk_label(r: &PatchRisk) -> &'static str {
    match r {
        PatchRisk::Safe => "safe",
        PatchRisk::Caution => "caution",
        PatchRisk::Risky => "risky",
    }
}
