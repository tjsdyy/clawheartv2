//! 通配发现：扫描用户主目录下所有 `~/.<agent>/skills/` 目录 + SSOT (`~/.agents/skills/`)
//!
//! Phase A：数据模型升级
//!  - 引入 SSOT 概念：所有 skill 优先认 `~/.agents/skills/` 中的真副本
//!  - 各 Agent 目录中的同名条目视为对 SSOT 的 binding（symlink → SSOT 视为"启用"）
//!  - 新增 `content_hash` 字段，借鉴 cc-switch，便于去重备份与更新检测
//!
//! 仍纯只读 — 写操作 (move_to_ssot / toggle / uninstall) 留待 Phase B。

use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::manifest;
use crate::security::sha256;

/// 系统级 dotfile — 不视为 Agent
const SYSTEM_DOTFILES: &[&str] = &[
    ".git", ".github", ".gitignore", ".gitconfig", ".gitattributes",
    ".vscode", ".idea", ".cache", ".cargo", ".rustup", ".npm", ".nvm",
    ".bashrc", ".zshrc", ".bash_history", ".zsh_history", ".profile",
    ".ssh", ".gnupg", ".local", ".config", ".docker", ".kube",
    ".DS_Store", ".Trash", ".CFUserTextEncoding",
    // ClawHeart 自身的数据目录 — 不应作为扫描对象（自检在 FP-008 中独立完成）
    ".clawheart", ".clawheart-v2", ".clawheartv2",
];

/// SSOT 默认目录（相对 home）— 借鉴 cc-switch 的 `.agents/skills/`，跨工具通用
const SSOT_REL: &str = ".agents/skills";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum BindingKind {
    /// Agent 目录中没有此 skill
    None,
    /// 真实文件 — Unmanaged（未纳入 SSOT 管理）
    Real { path: String, modified_unix: i64 },
    /// symlink 指向某处
    Symlink {
        path: String,
        target: String,
        points_to_ssot: bool,
    },
    /// symlink 但目标不存在
    Broken { path: String, target: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentBinding {
    pub agent_name: String,
    pub binding: BindingKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredSkill {
    /// 跨 Agent 唯一 id：标准化 slug（小写、连字符）
    pub id: String,
    /// 显示名（frontmatter.name 优先，回退目录名）
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,

    /// SSOT 状态
    pub in_ssot: bool,
    pub ssot_path: Option<String>,

    /// 每个发现 Agent 的绑定状态（agent_name → binding）
    pub bindings: Vec<AgentBinding>,

    /// 是否有 SKILL.md
    pub has_skill_md: bool,
    pub file_count: u32,
    pub total_bytes: u64,
    /// 内容哈希（前 16 字符 SHA256），借鉴 cc-switch
    pub content_hash: Option<String>,

    // ===  向后兼容：保留这两个字段供 Phase A 老 UI 使用 ===
    /// 第一个启用的 Agent；若全部未启用则取 SSOT；否则取第一个发现
    pub source_agent: String,
    /// 主路径：SSOT 优先；否则第一个绑定的 path
    pub source_path: String,
}

/// 返回当前 SSOT 绝对路径（即使尚未创建）
pub fn ssot_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(SSOT_REL))
}

pub fn discover_all() -> Vec<DiscoveredSkill> {
    let Some(home) = dirs::home_dir() else { return vec![] };
    let ssot = home.join(SSOT_REL);

    // ──────────────────────────────────────────────────────────────
    // Pass 1：SSOT 主副本
    // ──────────────────────────────────────────────────────────────
    let mut by_id: HashMap<String, DiscoveredSkill> = HashMap::new();

    if ssot.is_dir() {
        if let Ok(entries) = fs::read_dir(&ssot) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) else { continue };
                if dir_name.starts_with('.') {
                    continue;
                }
                if let Some(mut s) = inspect_skill(&path, "agents") {
                    s.in_ssot = true;
                    s.ssot_path = Some(path.to_string_lossy().into_owned());
                    s.bindings.clear(); // SSOT 是主目录，不算 binding
                    by_id.insert(s.id.clone(), s);
                }
            }
        }
    }

    // ──────────────────────────────────────────────────────────────
    // Pass 2：扫各 Agent 目录，合并到对应 id
    // ──────────────────────────────────────────────────────────────
    let Ok(home_entries) = fs::read_dir(&home) else { return by_id.into_values().collect() };
    for entry in home_entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if !dir_name.starts_with('.') || SYSTEM_DOTFILES.iter().any(|s| *s == dir_name) {
            continue;
        }
        let agent_name = dir_name.trim_start_matches('.').to_string();
        if agent_name == "agents" {
            continue; // SSOT 已在 Pass 1 处理
        }

        // 默认目录
        let skills_dir = path.join("skills");
        if skills_dir.is_dir() {
            scan_agent_dir(&skills_dir, &agent_name, &ssot, false, &mut by_id);
            continue;
        }

        // Cursor VSIX 适配
        if agent_name == "cursor" {
            let fallback = path.join("extensions");
            if fallback.is_dir() {
                scan_agent_dir(&fallback, &agent_name, &ssot, true, &mut by_id);
            }
        }
    }

    // 收尾：填 source_agent / source_path（向后兼容）
    let mut out: Vec<DiscoveredSkill> = by_id.into_values().collect();
    for s in &mut out {
        finalize_compat_fields(s);
    }
    out.sort_by(|a, b| {
        // Unmanaged 排前面（提醒用户迁入）；其次按 name
        let a_managed = a.in_ssot;
        let b_managed = b.in_ssot;
        match a_managed.cmp(&b_managed) {
            std::cmp::Ordering::Equal => a.name.cmp(&b.name),
            other => other,
        }
    });
    out
}

fn scan_agent_dir(
    dir: &Path,
    agent_name: &str,
    ssot: &Path,
    is_vsix: bool,
    by_id: &mut HashMap<String, DiscoveredSkill>,
) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() && entry.file_type().map(|t| !t.is_symlink()).unwrap_or(true) {
            continue;
        }
        let Some(dir_name) = p.file_name().and_then(|n| n.to_str()) else { continue };
        if dir_name.starts_with('.') {
            continue;
        }
        if is_vsix && !p.join("package.json").is_file() {
            continue;
        }

        let binding = classify_binding(&p, ssot);
        let id = canonicalize_id(dir_name);

        if let Some(existing) = by_id.get_mut(&id) {
            existing.bindings.push(AgentBinding {
                agent_name: agent_name.to_string(),
                binding,
            });
            continue;
        }

        // 不在 SSOT 中 → 新建条目（Unmanaged 或独立条目）
        let mut s = if is_vsix {
            match inspect_vsix(&p, agent_name) {
                Some(s) => s,
                None => continue,
            }
        } else {
            match inspect_skill(&p, agent_name) {
                Some(s) => s,
                None => continue,
            }
        };
        s.id = id.clone();
        s.bindings.push(AgentBinding {
            agent_name: agent_name.to_string(),
            binding,
        });
        by_id.insert(id, s);
    }
}

fn classify_binding(p: &Path, ssot: &Path) -> BindingKind {
    let path_str = p.to_string_lossy().into_owned();
    // 读 symlink
    if let Ok(meta) = fs::symlink_metadata(p) {
        if meta.file_type().is_symlink() {
            match fs::read_link(p) {
                Ok(target) => {
                    let resolved = if target.is_absolute() {
                        target.clone()
                    } else {
                        p.parent().map(|par| par.join(&target)).unwrap_or(target.clone())
                    };
                    let exists = resolved.exists();
                    let target_str = resolved.to_string_lossy().into_owned();
                    if !exists {
                        return BindingKind::Broken {
                            path: path_str,
                            target: target_str,
                        };
                    }
                    let points_to_ssot = resolved.starts_with(ssot);
                    return BindingKind::Symlink {
                        path: path_str,
                        target: target_str,
                        points_to_ssot,
                    };
                }
                Err(_) => {
                    return BindingKind::Broken {
                        path: path_str,
                        target: String::new(),
                    };
                }
            }
        }
    }
    // 真实文件 / 目录
    let modified_unix = fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    BindingKind::Real {
        path: path_str,
        modified_unix,
    }
}

/// id 标准化：小写 + 非字母数字转 '-' + 去除前后 '-'
fn canonicalize_id(dir_name: &str) -> String {
    let mut out = String::with_capacity(dir_name.len());
    let mut prev_dash = false;
    for c in dir_name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn finalize_compat_fields(s: &mut DiscoveredSkill) {
    // source_agent
    s.source_agent = if let Some(b) = s.bindings.iter().find(|b| {
        matches!(b.binding, BindingKind::Symlink { points_to_ssot: true, .. })
    }) {
        b.agent_name.clone()
    } else if s.in_ssot {
        "agents".to_string()
    } else if let Some(b) = s.bindings.first() {
        b.agent_name.clone()
    } else {
        "unknown".to_string()
    };
    // source_path
    s.source_path = if let Some(p) = &s.ssot_path {
        p.clone()
    } else if let Some(b) = s.bindings.first() {
        match &b.binding {
            BindingKind::Real { path, .. } => path.clone(),
            BindingKind::Symlink { path, .. } => path.clone(),
            BindingKind::Broken { path, .. } => path.clone(),
            BindingKind::None => String::new(),
        }
    } else {
        String::new()
    };
}

fn inspect_skill(skill_dir: &Path, agent: &str) -> Option<DiscoveredSkill> {
    let dir_name = skill_dir.file_name()?.to_str()?.to_string();
    if dir_name.starts_with('.') {
        return None;
    }

    let mut file_count: u32 = 0;
    let mut total_bytes: u64 = 0;
    walk_count(skill_dir, &mut file_count, &mut total_bytes, 0);
    if file_count == 0 {
        return None;
    }

    let skill_md = skill_dir.join("SKILL.md");
    let has_skill_md = skill_md.is_file();

    let mut name = dir_name.clone();
    let mut description: Option<String> = None;
    let mut version: Option<String> = None;

    if has_skill_md {
        if let Ok(text) = fs::read_to_string(&skill_md) {
            let fm = manifest::parse(&text);
            if let Some(n) = fm.name.filter(|s| !s.is_empty()) {
                name = n;
            }
            description = fm.description;
            version = fm.version;
        }
    } else {
        let pkg = skill_dir.join("package.json");
        if let Ok(text) = fs::read_to_string(&pkg) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(n) = v.get("name").and_then(|s| s.as_str()) {
                    name = n.to_string();
                }
                description = v.get("description").and_then(|s| s.as_str()).map(String::from);
                version = v.get("version").and_then(|s| s.as_str()).map(String::from);
            }
        }
    }

    let content_hash = compute_content_hash(skill_dir);
    let id = canonicalize_id(&dir_name);

    Some(DiscoveredSkill {
        id,
        name,
        description,
        version,
        in_ssot: false,
        ssot_path: None,
        bindings: Vec::new(),
        has_skill_md,
        file_count,
        total_bytes,
        content_hash,
        source_agent: agent.to_string(),
        source_path: skill_dir.to_string_lossy().into_owned(),
    })
}

/// Cursor VSIX 适配
fn inspect_vsix(dir: &Path, agent: &str) -> Option<DiscoveredSkill> {
    let dir_name = dir.file_name()?.to_str()?.to_string();
    let pkg_json = dir.join("package.json");
    if !pkg_json.is_file() {
        return None;
    }
    let mut file_count: u32 = 0;
    let mut total_bytes: u64 = 0;
    walk_count(dir, &mut file_count, &mut total_bytes, 0);

    let mut name = dir_name.clone();
    let mut description: Option<String> = None;
    let mut version: Option<String> = None;

    if let Ok(text) = fs::read_to_string(&pkg_json) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            let display = v.get("displayName").and_then(|s| s.as_str()).map(String::from);
            let raw_name = v.get("name").and_then(|s| s.as_str()).map(String::from);
            let publisher = v.get("publisher").and_then(|s| s.as_str()).map(String::from);
            name = match (display, raw_name, publisher) {
                (Some(d), _, Some(p)) => format!("{} · {}", p, d),
                (Some(d), _, None) => d,
                (None, Some(n), Some(p)) => format!("{}.{}", p, n),
                (None, Some(n), None) => n,
                _ => dir_name.clone(),
            };
            description = v.get("description").and_then(|s| s.as_str()).map(String::from);
            version = v.get("version").and_then(|s| s.as_str()).map(String::from);
        }
    }

    let content_hash = compute_content_hash(dir);
    let id = canonicalize_id(&format!("ext-{}", dir_name));

    Some(DiscoveredSkill {
        id,
        name,
        description,
        version,
        in_ssot: false,
        ssot_path: None,
        bindings: Vec::new(),
        has_skill_md: false,
        file_count,
        total_bytes,
        content_hash,
        source_agent: agent.to_string(),
        source_path: dir.to_string_lossy().into_owned(),
    })
}

fn walk_count(dir: &Path, count: &mut u32, bytes: &mut u64, depth: u32) {
    if depth > 6 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
            if n == ".git" || n == "node_modules" || n == ".DS_Store" {
                continue;
            }
        }
        if p.is_dir() {
            walk_count(&p, count, bytes, depth + 1);
        } else if let Ok(m) = p.metadata() {
            *count = count.saturating_add(1);
            *bytes = bytes.saturating_add(m.len());
        }
    }
}

/// 计算目录内容哈希 — 仅前 16 字符 SHA256（足够去重）
///
/// 算法：递归收集所有相对路径 + 文件内容 → sha256 → 取前 16 字符
fn compute_content_hash(dir: &Path) -> Option<String> {
    let mut buf: Vec<u8> = Vec::new();
    collect_for_hash(dir, dir, &mut buf, 0).ok()?;
    let full = sha256::hex_string_bytes(&buf);
    Some(full.chars().take(16).collect())
}

fn collect_for_hash(root: &Path, dir: &Path, buf: &mut Vec<u8>, depth: u32) -> std::io::Result<()> {
    if depth > 6 {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let p = entry.path();
        if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
            if n == ".git" || n == "node_modules" || n == ".DS_Store" {
                continue;
            }
        }
        let rel = p.strip_prefix(root).unwrap_or(&p);
        buf.extend_from_slice(rel.to_string_lossy().as_bytes());
        buf.push(b'\n');
        if p.is_dir() {
            collect_for_hash(root, &p, buf, depth + 1)?;
        } else if p.is_file() {
            // 大文件只取大小 + 修改时间，避免读 GB 级
            let meta = p.metadata()?;
            if meta.len() > 1024 * 1024 {
                buf.extend_from_slice(format!("[size:{}]", meta.len()).as_bytes());
            } else if let Ok(bytes) = fs::read(&p) {
                buf.extend_from_slice(&bytes);
            }
            buf.push(b'\n');
        }
    }
    Ok(())
}

pub fn find_by_id(id: &str) -> Option<DiscoveredSkill> {
    discover_all().into_iter().find(|s| s.id == id)
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillFile {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillDetail {
    pub meta: DiscoveredSkill,
    pub files: Vec<SkillFile>,
    pub skill_md: Option<String>,
    pub readme: Option<String>,
}

const SKILL_MD_MAX: usize = 64 * 1024;

pub fn detail(id: &str) -> Option<SkillDetail> {
    let meta = find_by_id(id)?;
    let root_path = if meta.in_ssot {
        meta.ssot_path.clone().unwrap_or_else(|| meta.source_path.clone())
    } else {
        meta.source_path.clone()
    };
    let root = PathBuf::from(&root_path);
    let mut files: Vec<SkillFile> = Vec::new();
    walk_list(&root, &root, &mut files, 0);
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let skill_md = fs::read_to_string(root.join("SKILL.md"))
        .ok()
        .map(|s| truncate(s, SKILL_MD_MAX));

    let readme = ["README.md", "readme.md", "Readme.md"]
        .iter()
        .find_map(|name| fs::read_to_string(root.join(name)).ok())
        .map(|s| truncate(s, SKILL_MD_MAX));

    Some(SkillDetail {
        meta,
        files,
        skill_md,
        readme,
    })
}

fn walk_list(root: &Path, dir: &Path, out: &mut Vec<SkillFile>, depth: u32) {
    if depth > 6 || out.len() >= 500 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
            if n == ".git" || n == "node_modules" || n == ".DS_Store" {
                continue;
            }
        }
        let rel = match p.strip_prefix(root) {
            Ok(r) => r.to_string_lossy().into_owned(),
            Err(_) => continue,
        };
        if p.is_dir() {
            out.push(SkillFile {
                path: rel,
                size: 0,
                is_dir: true,
                depth,
            });
            walk_list(root, &p, out, depth + 1);
        } else if let Ok(meta) = p.metadata() {
            out.push(SkillFile {
                path: rel,
                size: meta.len(),
                is_dir: false,
                depth,
            });
        }
    }
}

fn truncate(s: String, max: usize) -> String {
    if s.len() <= max {
        s
    } else {
        let mut t = s;
        t.truncate(max);
        t.push_str("\n\n…（已截断，仅显示前 64KB）");
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_discover() {
        // 在 CI 上 home dir 通常无 .agents/skills，应返回空 vec 而非 panic
        let _ = discover_all();
    }

    #[test]
    fn canonicalize_id_basic() {
        assert_eq!(canonicalize_id("Web-Fetch"), "web-fetch");
        assert_eq!(canonicalize_id("foo bar"), "foo-bar");
        assert_eq!(canonicalize_id("foo__bar"), "foo-bar");
        assert_eq!(canonicalize_id("---foo"), "foo");
        assert_eq!(canonicalize_id("@scope/name"), "scope-name");
    }
}
