//! 把选中的技能打包为 zip。
//!
//! 结构：
//! ```text
//! clawheart-skills-backup-<ts>.zip
//! ├── manifest.json
//! └── <agent>/<skill_dir>/...
//! ```

use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::CompressionMethod;

use super::discover::{discover_all, DiscoveredSkill};

#[derive(Debug, Clone, Serialize)]
pub struct BackupResult {
    pub zip_path: String,
    pub skill_count: u32,
    pub total_bytes: u64,
}

/// 排除规则 — 不写入 zip 的文件/目录名
const SKIP: &[&str] = &[
    ".git",
    "node_modules",
    ".DS_Store",
    ".env",
    ".env.local",
    "credentials.json",
];

/// 把 `ids` 对应的技能打包到 `output_zip_path`。
///
/// `output_zip_path` 是绝对路径（前端通过 dialog 选好后传入）。
pub fn backup_skills(ids: &[String], output_zip_path: &Path) -> io::Result<BackupResult> {
    let all = discover_all();
    let selected: Vec<&DiscoveredSkill> = all.iter().filter(|s| ids.iter().any(|i| i == &s.id)).collect();

    if selected.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "no matching skills"));
    }

    if let Some(parent) = output_zip_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let file = File::create(output_zip_path)?;
    let mut zip = ZipWriter::new(BufWriter::new(file));
    let opts =
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated).unix_permissions(0o644);

    let mut skill_count: u32 = 0;
    let mut total_bytes: u64 = 0;

    // 1. manifest.json
    let manifest = serde_json::json!({
        "format": "clawheart-skills-backup/v1",
        "created_at": chrono_like_ts(),
        "skills": selected.iter().map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "version": s.version,
            "source_agent": s.source_agent,
            "source_path": s.source_path,
            "file_count": s.file_count,
            "total_bytes": s.total_bytes,
            "has_skill_md": s.has_skill_md,
        })).collect::<Vec<_>>()
    });
    zip.start_file("manifest.json", opts)?;
    zip.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;

    // 2. 每个 skill 完整复制
    for skill in &selected {
        let src_root = Path::new(&skill.source_path);
        let dir_name = src_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let zip_prefix = format!("{}/{}", skill.source_agent, dir_name);
        add_dir_to_zip(&mut zip, src_root, &zip_prefix, opts, &mut total_bytes)?;
        skill_count = skill_count.saturating_add(1);
    }

    zip.finish()?;

    Ok(BackupResult {
        zip_path: output_zip_path.to_string_lossy().into_owned(),
        skill_count,
        total_bytes,
    })
}

fn add_dir_to_zip<W: Write + io::Seek>(
    zip: &mut ZipWriter<W>,
    src_root: &Path,
    zip_prefix: &str,
    opts: SimpleFileOptions,
    total_bytes: &mut u64,
) -> io::Result<()> {
    let mut stack: Vec<PathBuf> = vec![src_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if SKIP.iter().any(|s| *s == name) {
                    continue;
                }
            }
            let rel = match path.strip_prefix(src_root) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let zip_path = if rel.as_os_str().is_empty() {
                zip_prefix.to_string()
            } else {
                format!("{}/{}", zip_prefix, rel.to_string_lossy().replace('\\', "/"))
            };

            if path.is_dir() {
                // zip 目录条目（可省略，但显式更兼容）
                let dir_entry = format!("{}/", zip_path);
                let _ = zip.add_directory(dir_entry, opts);
                stack.push(path);
            } else if path.is_file() {
                let mut f = File::open(&path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                *total_bytes = total_bytes.saturating_add(buf.len() as u64);
                zip.start_file(zip_path, opts)?;
                zip.write_all(&buf)?;
            }
        }
    }
    Ok(())
}

/// 简易 ISO-ish 时间戳（避免引入 chrono）— 仅用于 manifest 标记，精度足够
fn chrono_like_ts() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch:{}", secs)
}
