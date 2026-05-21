//! SSOT 写操作（Phase B）：
//! - `ensure_ssot_dir` 确保 `~/.agents/skills/` 存在
//! - `move_to_ssot` 把 unmanaged skill 迁入 SSOT，原位置改为 symlink
//! - `toggle_binding` 创建/移除某 Agent 目录的 symlink → SSOT
//! - `uninstall_skill` 删除 SSOT 主体 + 清所有 symlink
//! - `repair_binding` 修复 broken / 指向非 SSOT 的 symlink
//!
//! 安全策略（用户决策 2/3/5）：
//! - 直接写，无 dry-run 沙箱
//! - 仅 symlink，不 fallback copy；Windows 无 dev mode 权限时返回错误
//! - 每次危险操作前自动 zip 备份原位置内容到 `~/.clawheart-v2/auto-backups/skills/<ts>-<op>.zip`

use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::discover::{self, DiscoveredSkill};

/// 自动备份目录（相对 home）
const AUTO_BACKUP_REL: &str = ".clawheart-v2/auto-backups/skills";

#[derive(Debug, Clone, Serialize)]
pub struct SsotConfig {
    pub path: String,
    pub exists: bool,
    pub total_skills: u32,
    pub total_bytes: u64,
}

/// 返回 SSOT 配置（路径 + 是否存在 + 主副本数量/字节数）
pub fn ssot_config() -> SsotConfig {
    let Some(path) = discover::ssot_path() else {
        return SsotConfig {
            path: String::new(),
            exists: false,
            total_skills: 0,
            total_bytes: 0,
        };
    };
    let exists = path.is_dir();
    let (total_skills, total_bytes) = if exists {
        count_top_level(&path)
    } else {
        (0, 0)
    };
    SsotConfig {
        path: path.to_string_lossy().into_owned(),
        exists,
        total_skills,
        total_bytes,
    }
}

fn count_top_level(dir: &Path) -> (u32, u64) {
    let Ok(entries) = fs::read_dir(dir) else { return (0, 0) };
    let mut count: u32 = 0;
    let mut bytes: u64 = 0;
    for e in entries.flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        if p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(true)
        {
            continue;
        }
        count = count.saturating_add(1);
        bytes = bytes.saturating_add(dir_size(&p));
    }
    (count, bytes)
}

fn dir_size(dir: &Path) -> u64 {
    let mut total: u64 = 0;
    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(m) = p.metadata() {
                total = total.saturating_add(m.len());
            }
        }
    }
    total
}

pub fn ensure_ssot_dir() -> io::Result<PathBuf> {
    let path = discover::ssot_path().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "无法定位 home 目录")
    })?;
    fs::create_dir_all(&path)?;
    Ok(path)
}

// ──────────────────────────────────────────────────────────────────
// move_to_ssot
// ──────────────────────────────────────────────────────────────────

/// 把一个 unmanaged skill 迁入 SSOT。
///
/// 步骤：
/// 1. 确保 SSOT 目录存在
/// 2. 自动 zip 备份原 skill 目录
/// 3. 复制源目录 → `<SSOT>/<id>/`
/// 4. 删除源目录
/// 5. 在源位置创建 symlink → SSOT
///
/// 若 skill 已在 SSOT，返回错误（不要重复迁入）。
pub fn move_to_ssot(id: &str) -> io::Result<DiscoveredSkill> {
    let skill = discover::find_by_id(id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("skill `{}` 不存在", id))
    })?;

    if skill.in_ssot {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("skill `{}` 已在 SSOT 中", id),
        ));
    }

    // 找到第一个 Real binding（unmanaged 源）
    let source_path = first_real_binding_path(&skill).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "未找到可迁入的真实文件（所有 Agent binding 都是 symlink 或缺失）",
        )
    })?;
    let source = PathBuf::from(&source_path);

    let ssot_dir = ensure_ssot_dir()?;
    let target = ssot_dir.join(&skill.id);

    if target.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("SSOT 中已有同名目录 `{}`", target.display()),
        ));
    }

    // Step 2: auto-backup
    auto_backup(&source, &skill.id, "move_to_ssot")?;

    // Step 3: copy
    copy_dir(&source, &target)?;

    // Step 4: remove source
    fs::remove_dir_all(&source)?;

    // Step 5: symlink
    create_symlink(&target, &source)?;

    // 重新发现以拿到新状态
    discover::find_by_id(id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "迁入后未能重新发现 skill")
    })
}

fn first_real_binding_path(skill: &DiscoveredSkill) -> Option<String> {
    for b in &skill.bindings {
        if let discover::BindingKind::Real { path, .. } = &b.binding {
            return Some(path.clone());
        }
    }
    None
}

// ──────────────────────────────────────────────────────────────────
// toggle_binding
// ──────────────────────────────────────────────────────────────────

/// 在某 Agent 目录中创建（或移除）指向 SSOT 的 symlink。
///
/// `enabled = true`：
/// - 要求 skill 已在 SSOT
/// - 在 `~/.<agent>/skills/<id>` 处创建 symlink → `<SSOT>/<id>`
/// - 若该路径已存在（real / 别处 symlink），返回错误（必须先 repair 或 move）
///
/// `enabled = false`：
/// - 仅当该位置是 symlink → SSOT 时移除
/// - 否则返回错误，避免误删 real 文件
pub fn toggle_binding(id: &str, agent: &str, enabled: bool) -> io::Result<DiscoveredSkill> {
    let skill = discover::find_by_id(id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("skill `{}` 不存在", id))
    })?;
    if !skill.in_ssot {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("skill `{}` 不在 SSOT 中；请先「移入 SSOT」", id),
        ));
    }
    let ssot_target = skill.ssot_path.clone().ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "skill 标记 in_ssot 但缺 ssot_path")
    })?;
    let ssot_target = PathBuf::from(ssot_target);

    let agent_dir = agent_skills_dir(agent)?;
    fs::create_dir_all(&agent_dir)?;
    let link_path = agent_dir.join(&skill.id);

    if enabled {
        // 创建 symlink
        if link_path.exists() || fs::symlink_metadata(&link_path).is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "`{}` 已存在 · 请先卸载或在「详情」中修复",
                    link_path.display()
                ),
            ));
        }
        create_symlink(&ssot_target, &link_path)?;
    } else {
        // 仅移除 SSOT-symlink
        let meta = match fs::symlink_metadata(&link_path) {
            Ok(m) => m,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("`{}` 不存在，无需移除", link_path.display()),
                ));
            }
        };
        if !meta.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "`{}` 是真实文件而非 symlink，拒绝删除（避免数据丢失）",
                    link_path.display()
                ),
            ));
        }
        // 二次校验：target 必须指向 SSOT 内
        match fs::read_link(&link_path) {
            Ok(target) => {
                let resolved = if target.is_absolute() {
                    target
                } else {
                    link_path
                        .parent()
                        .map(|p| p.join(&target))
                        .unwrap_or(target)
                };
                if !resolved.starts_with(&ssot_target.parent().unwrap_or(Path::new("/"))) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "`{}` 指向 SSOT 之外（{}），拒绝自动删除",
                            link_path.display(),
                            resolved.display()
                        ),
                    ));
                }
            }
            Err(_) => {
                // broken symlink — 允许删
            }
        }
        fs::remove_file(&link_path)?;
    }

    discover::find_by_id(id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "toggle 后未能重新发现 skill")
    })
}

fn agent_skills_dir(agent: &str) -> io::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "无 home dir"))?;
    if agent.contains('/') || agent.starts_with('.') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("非法 agent 名 `{}`", agent),
        ));
    }
    Ok(home.join(format!(".{}/skills", agent)))
}

// ──────────────────────────────────────────────────────────────────
// uninstall_skill
// ──────────────────────────────────────────────────────────────────

/// 卸载 skill：
/// 1. 自动 zip 备份 SSOT 主体
/// 2. 删除所有 Agent 目录中指向 SSOT 的 symlink
/// 3. 删除 SSOT 主体
pub fn uninstall_skill(id: &str) -> io::Result<()> {
    let skill = discover::find_by_id(id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("skill `{}` 不存在", id))
    })?;
    if !skill.in_ssot {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "skill 不在 SSOT，无法用此命令卸载（请用文件管理器手动处理 unmanaged 副本）",
        ));
    }
    let ssot_path = skill
        .ssot_path
        .clone()
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "缺 ssot_path"))?;

    auto_backup(&ssot_path, &skill.id, "uninstall")?;

    // 清各 Agent symlink（仅删 → SSOT 的）
    for b in &skill.bindings {
        if let discover::BindingKind::Symlink {
            path,
            points_to_ssot: true,
            ..
        } = &b.binding
        {
            let _ = fs::remove_file(path);
        }
    }

    fs::remove_dir_all(&ssot_path)?;
    Ok(())
}

// ──────────────────────────────────────────────────────────────────
// repair_binding
// ──────────────────────────────────────────────────────────────────

/// 修复 broken / external symlink：删除现有 link，重建指向 SSOT
pub fn repair_binding(id: &str, agent: &str) -> io::Result<DiscoveredSkill> {
    let skill = discover::find_by_id(id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("skill `{}` 不存在", id))
    })?;
    if !skill.in_ssot {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "skill 不在 SSOT，无法修复",
        ));
    }
    let ssot_target = skill
        .ssot_path
        .clone()
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "缺 ssot_path"))?;

    let agent_dir = agent_skills_dir(agent)?;
    let link_path = agent_dir.join(&skill.id);

    // 仅当现有项是 symlink 时才覆盖；real 文件需要用户手动迁移
    if let Ok(meta) = fs::symlink_metadata(&link_path) {
        if !meta.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "`{}` 是真实文件而非 symlink，拒绝自动覆盖",
                    link_path.display()
                ),
            ));
        }
        let _ = fs::remove_file(&link_path);
    }
    fs::create_dir_all(&agent_dir)?;
    create_symlink(&ssot_target, &link_path)?;

    discover::find_by_id(id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "repair 后未能重新发现 skill")
    })
}

// ──────────────────────────────────────────────────────────────────
// helpers
// ──────────────────────────────────────────────────────────────────

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> io::Result<()> {
    // Windows 需要 SeCreateSymbolicLinkPrivilege（开发者模式）或管理员
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
    .map_err(|e| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Windows 创建 symlink 失败（需开发者模式或管理员）：{}",
                e
            ),
        )
    })
}

fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let to = dst.join(&name);
        if let Some(s) = name.to_str() {
            if s == ".DS_Store" {
                continue;
            }
        }
        if path.is_dir() {
            copy_dir(&path, &to)?;
        } else if path.is_file() {
            fs::copy(&path, &to)?;
        }
    }
    Ok(())
}

/// auto-backup：把目录打包到 `~/.clawheart-v2/auto-backups/skills/<ts>-<id>-<op>.zip`
///
/// 失败不阻塞主流程（写日志后返回 Ok），不让备份失败破坏迁入/卸载本身
fn auto_backup(dir: &Path, id: &str, op: &str) -> io::Result<()> {
    let Some(home) = dirs::home_dir() else { return Ok(()) };
    let backup_dir = home.join(AUTO_BACKUP_REL);
    if fs::create_dir_all(&backup_dir).is_err() {
        return Ok(());
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let safe_id: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    let zip_path = backup_dir.join(format!("{}-{}-{}.zip", ts, safe_id, op));

    if zip_dir(dir, &zip_path).is_err() {
        tracing::warn!(?dir, ?zip_path, "auto-backup zip 失败（已忽略）");
    }
    Ok(())
}

fn zip_dir(src: &Path, zip_path: &Path) -> io::Result<()> {
    use std::fs::File;
    use std::io::{BufWriter, Read, Write};
    use zip::write::{SimpleFileOptions, ZipWriter};
    use zip::CompressionMethod;

    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(BufWriter::new(file));
    let opts = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let mut stack: Vec<PathBuf> = vec![src.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(rel) = path.strip_prefix(src) else { continue };
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if path.is_dir() {
                let _ = zip.add_directory(format!("{}/", rel_str), opts);
                stack.push(path);
            } else if path.is_file() {
                let mut f = File::open(&path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.start_file(rel_str, opts)?;
                zip.write_all(&buf)?;
            }
        }
    }
    zip.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssot_config_smoke() {
        let cfg = ssot_config();
        assert!(!cfg.path.is_empty());
        // 不强求存在
    }

    #[test]
    fn agent_skills_dir_rejects_dots() {
        assert!(agent_skills_dir("..").is_err());
        assert!(agent_skills_dir("a/b").is_err());
        assert!(agent_skills_dir(".claude").is_err());
    }
}
