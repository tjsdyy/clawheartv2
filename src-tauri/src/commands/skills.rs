//! IPC: 技能治理
use crate::error::{AppError, AppResult};
use crate::skills::{
    backup as skill_backup, discover as skill_discover, manage as skill_manage, DiscoveredSkill,
};
use crate::state::AppState;
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

#[derive(Serialize)]
pub struct SkillListItem {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub safety_label: String,
    pub scan_score: u32,
    pub user_enabled: bool,
    pub system_status: String,
}

#[tauri::command]
pub fn list_skills(state: State<AppState>) -> AppResult<Vec<SkillListItem>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            if let Ok(rows) = crate::storage::queries::skills::list(db) {
                return Ok(rows
                    .into_iter()
                    .map(|s| SkillListItem {
                        slug: s.slug,
                        name: s.name,
                        description: s.description,
                        version: s.version,
                        safety_label: s.safety_label,
                        scan_score: s.scan_score.max(0) as u32,
                        user_enabled: s.user_enabled,
                        system_status: s.system_status,
                    })
                    .collect());
            }
        }
    }
    let _ = state;
    Ok(vec![])
}

#[tauri::command]
pub fn toggle_skill(state: State<AppState>, slug: String, enabled: bool) -> AppResult<()> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            crate::storage::queries::skills::toggle(db, &slug, enabled)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(());
        }
    }
    let _ = (state, slug, enabled);
    Ok(())
}

#[tauri::command]
pub fn set_skill_safety(
    _state: State<AppState>,
    _slug: String,
    _label: String,
) -> AppResult<()> {
    // W17 真实实现：UPDATE skills SET safety_label = ?2 WHERE slug = ?1
    // 当前 stub 等待用户审核流程定义
    Ok(())
}

#[tauri::command]
pub fn scan_skill(_slug: String) -> AppResult<serde_json::Value> {
    // skill_scanner 已有 72 条规则，W17 接入完整扫描；当前返回乐观结果
    Ok(serde_json::json!({ "score": 100, "blocked": false, "findings": [] }))
}

#[tauri::command]
pub fn sync_skills() -> AppResult<u32> {
    // W17 接云端技能库
    Ok(0)
}

// ──────────────────────────────────────────────────────────────────
// 技能备份 / 鉴定（W19）
// 通配扫描 ~/.<agent>/skills/ 自动捕获任何 Agent
// ──────────────────────────────────────────────────────────────────

/// 发现本机所有 Agent 安装的技能
#[tauri::command]
pub fn discover_local_skills() -> AppResult<Vec<DiscoveredSkill>> {
    Ok(skill_discover::discover_all())
}

/// 获取单个技能的详情：文件树 + SKILL.md / README 全文
#[tauri::command]
pub fn get_local_skill_detail(id: String) -> AppResult<skill_discover::SkillDetail> {
    skill_discover::detail(&id).ok_or_else(|| AppError::Other(format!("技能 {} 未找到", id)))
}

/// 对指定技能做安全鉴定扫描（用 SkillGuard 规则集）
#[tauri::command]
pub fn scan_local_skill(id: String) -> AppResult<serde_json::Value> {
    use crate::security::skill_scanner::{scan, Context, SkillBundle};
    use std::fs;

    let Some(skill) = skill_discover::find_by_id(&id) else {
        return Err(AppError::Other(format!("技能 {} 未找到", id)));
    };

    // 读取技能目录里的关键文件作为 bundle
    let root = std::path::Path::new(&skill.source_path);
    let manifest_text = fs::read_to_string(root.join("SKILL.md"))
        .or_else(|_| fs::read_to_string(root.join("package.json")))
        .unwrap_or_default();

    let mut files: Vec<(String, String, Context)> = Vec::new();
    collect_scan_targets(root, root, &mut files, 0);

    let bundle = SkillBundle {
        manifest: &manifest_text,
        files: files
            .iter()
            .map(|(n, c, ctx)| (n.as_str(), c.as_str(), *ctx))
            .collect(),
    };
    let report = scan(&bundle);

    Ok(serde_json::json!({
        "id": skill.id,
        "name": skill.name,
        "score": report.score,
        "blocked": report.blocked,
        "hard_triggers": report.hard_triggers,
        "findings": report.findings,
    }))
}

fn collect_scan_targets(
    root: &std::path::Path,
    dir: &std::path::Path,
    out: &mut Vec<(String, String, crate::security::skill_scanner::Context)>,
    depth: u32,
) {
    use crate::security::skill_scanner::Context;
    if depth > 4 || out.len() >= 50 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
            if n.starts_with('.') || n == "node_modules" {
                continue;
            }
        }
        if p.is_dir() {
            collect_scan_targets(root, &p, out, depth + 1);
        } else if p.is_file() {
            let Some(name) = p.file_name().and_then(|n| n.to_str()) else { continue };
            // 文件大小限制 256KB，避免大文件
            let Ok(meta) = p.metadata() else { continue };
            if meta.len() > 256 * 1024 {
                continue;
            }
            let ctx = if name.ends_with(".md") || name.ends_with(".json") || name.ends_with(".toml") {
                Context::Mention
            } else {
                Context::Exec
            };
            if let Ok(content) = std::fs::read_to_string(&p) {
                let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().into_owned();
                out.push((rel, content, ctx));
            }
        }
    }
}

/// 把选中技能打包为 zip
///
/// `output_zip_path` 为 None 时默认写到 `~/Downloads/clawheart-skills-backup-<epoch>.zip`
/// 成功后写入 `skill_backups` 表以供「备份历史」Tab 展示。
#[tauri::command]
pub fn backup_local_skills(
    state: State<AppState>,
    ids: Vec<String>,
    output_zip_path: Option<String>,
) -> AppResult<skill_backup::BackupResult> {
    if ids.is_empty() {
        return Err(AppError::Other("未选择任何技能".into()));
    }
    let out_path = match output_zip_path.filter(|s| !s.trim().is_empty()) {
        Some(p) => PathBuf::from(p),
        None => default_backup_path()?,
    };

    // 先发现一次，用于写入名称快照
    let all = skill_discover::discover_all();
    let selected_names: Vec<String> = ids
        .iter()
        .filter_map(|id| all.iter().find(|s| &s.id == id))
        .map(|s| s.name.clone())
        .collect();

    let result = skill_backup::backup_skills(&ids, &out_path)
        .map_err(|e| AppError::Other(format!("打包失败: {}", e)))?;

    #[cfg(feature = "storage")]
    if let Some(db) = &state.db {
        let ids_json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
        let names_json =
            serde_json::to_string(&selected_names).unwrap_or_else(|_| "[]".to_string());
        let _ = crate::storage::queries::skills::insert_backup(
            db,
            &result.zip_path,
            result.skill_count,
            result.total_bytes,
            &ids_json,
            &names_json,
        );
    }
    #[cfg(not(feature = "storage"))]
    let _ = (state, &selected_names);

    Ok(result)
}

#[derive(Serialize)]
pub struct SkillBackupItem {
    pub id: i64,
    pub created_at: String,
    pub zip_path: String,
    pub skill_count: i64,
    pub total_bytes: i64,
    pub skill_ids: Vec<String>,
    pub skill_names: Vec<String>,
    pub zip_exists: bool,
}

#[tauri::command]
pub fn list_skill_backups(state: State<AppState>) -> AppResult<Vec<SkillBackupItem>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let rows = crate::storage::queries::skills::list_backups(db, 100)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;

            return Ok(rows
                .into_iter()
                .map(|r| {
                    // 在返回前刷新 zip_exists（用户可能在外部删了 zip）
                    let exists_on_disk = std::path::Path::new(&r.zip_path).exists();
                    if r.zip_exists && !exists_on_disk {
                        let _ = crate::storage::queries::skills::mark_zip_missing(db, r.id);
                    }
                    SkillBackupItem {
                        id: r.id,
                        created_at: r.created_at,
                        zip_path: r.zip_path,
                        skill_count: r.skill_count,
                        total_bytes: r.total_bytes,
                        skill_ids: serde_json::from_str(&r.skill_ids).unwrap_or_default(),
                        skill_names: serde_json::from_str(&r.skill_names).unwrap_or_default(),
                        zip_exists: exists_on_disk && r.zip_exists,
                    }
                })
                .collect());
        }
    }
    let _ = state;
    Ok(vec![])
}

#[tauri::command]
pub fn delete_skill_backup(state: State<AppState>, id: i64) -> AppResult<()> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            crate::storage::queries::skills::delete_backup(db, id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(());
        }
    }
    let _ = (state, id);
    Ok(())
}

// ──────────────────────────────────────────────────────────────────
// SSOT 管理（Phase B）
// ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_ssot_config() -> AppResult<skill_manage::SsotConfig> {
    Ok(skill_manage::ssot_config())
}

/// 返回 auto-backup 目录绝对路径（即使尚未创建）
#[tauri::command]
pub fn get_skill_backup_dir() -> AppResult<String> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Other("无法定位 home 目录".into()))?;
    Ok(home.join(".clawheart-v2/auto-backups/skills").to_string_lossy().into_owned())
}

#[tauri::command]
pub fn ensure_ssot() -> AppResult<skill_manage::SsotConfig> {
    skill_manage::ensure_ssot_dir()
        .map_err(|e| AppError::Other(format!("创建 SSOT 失败：{}", e)))?;
    Ok(skill_manage::ssot_config())
}

#[tauri::command]
pub fn move_skill_to_ssot(id: String) -> AppResult<DiscoveredSkill> {
    skill_manage::move_to_ssot(&id)
        .map_err(|e| AppError::Other(format!("迁入失败：{}", e)))
}

#[tauri::command]
pub fn toggle_skill_binding(
    id: String,
    agent: String,
    enabled: bool,
) -> AppResult<DiscoveredSkill> {
    skill_manage::toggle_binding(&id, &agent, enabled)
        .map_err(|e| AppError::Other(format!("切换失败：{}", e)))
}

#[tauri::command]
pub fn uninstall_skill(id: String) -> AppResult<()> {
    skill_manage::uninstall_skill(&id)
        .map_err(|e| AppError::Other(format!("卸载失败：{}", e)))
}

#[tauri::command]
pub fn repair_skill_binding(id: String, agent: String) -> AppResult<DiscoveredSkill> {
    skill_manage::repair_binding(&id, &agent)
        .map_err(|e| AppError::Other(format!("修复失败：{}", e)))
}

fn default_backup_path() -> AppResult<PathBuf> {
    let downloads = dirs::download_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| AppError::Other("无法确定下载目录".into()))?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(downloads.join(format!("clawheart-skills-backup-{}.zip", ts)))
}
