//! 平台识别共用工具：配置文件存在性 + 可解析校验 + 关键字段验证。
//!
//! 设计原则：
//! - 判定单元是「配置文件可解析」，不是「目录是否存在」，降低假阳性
//! - 不引入新依赖（toml/yaml 用 lookalike 文本判定，对识别"是否安装"够用）
//! - 首匹配胜出 (first-match-wins)：候选按优先级排列
//! - 三态：Active / ConfigBroken / NotFound，映射到 DiscoveredAgent.status

use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    /// 严格 JSON
    Json,
    /// 容忍 `//` 行注释 + 尾逗号（适用 .mcp.json 类 JSONC）
    JsonRelaxed,
    /// TOML（无 crate，用文本 lookalike 判 key=value）
    Toml,
    /// .env（KEY=VALUE 行）
    DotEnv,
    /// YAML（无 crate，文本 lookalike）
    Yaml,
    /// 不解析，只判文件存在且非空
    Any,
}

#[derive(Debug, Clone)]
pub struct ConfigCandidate {
    pub path: PathBuf,
    pub format: ConfigFormat,
    /// 可选：解析后必须含此 key（dot path，如 "models.providers"）才算 Active
    pub required_key: Option<String>,
}

impl ConfigCandidate {
    pub fn new(path: PathBuf, format: ConfigFormat) -> Self {
        Self { path, format, required_key: None }
    }
    pub fn require_key(mut self, key: impl Into<String>) -> Self {
        self.required_key = Some(key.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionStatus {
    /// 配置文件可解析（且字段满足）
    Active,
    /// 文件存在但解析失败 / 关键字段缺失
    ConfigBroken,
    /// 没有任何候选文件存在
    NotFound,
}

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub status: DetectionStatus,
    /// 第一个命中（Active 优先；否则 ConfigBroken）的路径
    pub matched_path: Option<PathBuf>,
}

impl DetectionResult {
    pub fn is_present(&self) -> bool {
        self.status != DetectionStatus::NotFound
    }
}

/// 按候选顺序检测：返回第一个 Active；否则返回首个 ConfigBroken；否则 NotFound。
pub fn detect_config(candidates: &[ConfigCandidate]) -> DetectionResult {
    let mut first_broken: Option<PathBuf> = None;
    for c in candidates {
        if !c.path.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&c.path) else {
            continue;
        };
        if content.trim().is_empty() {
            if first_broken.is_none() {
                first_broken = Some(c.path.clone());
            }
            continue;
        }
        let ok = match c.format {
            ConfigFormat::Any => true,
            ConfigFormat::Json => parse_strict_json(&content)
                .map(|v| has_required_key_json(&v, c.required_key.as_deref()))
                .unwrap_or(false),
            ConfigFormat::JsonRelaxed => parse_relaxed_json(&content)
                .map(|v| has_required_key_json(&v, c.required_key.as_deref()))
                .unwrap_or(false),
            ConfigFormat::DotEnv => check_dotenv(&content, c.required_key.as_deref()),
            ConfigFormat::Toml => check_toml_lookalike(&content, c.required_key.as_deref()),
            ConfigFormat::Yaml => check_yaml_lookalike(&content, c.required_key.as_deref()),
        };
        if ok {
            return DetectionResult {
                status: DetectionStatus::Active,
                matched_path: Some(c.path.clone()),
            };
        } else if first_broken.is_none() {
            first_broken = Some(c.path.clone());
        }
    }
    if let Some(p) = first_broken {
        DetectionResult { status: DetectionStatus::ConfigBroken, matched_path: Some(p) }
    } else {
        DetectionResult { status: DetectionStatus::NotFound, matched_path: None }
    }
}

/// 把 DetectionStatus 映射到 DiscoveredAgent.status 字段值
pub fn status_label(s: &DetectionStatus) -> &'static str {
    match s {
        DetectionStatus::Active => "active",
        DetectionStatus::ConfigBroken => "config_broken",
        DetectionStatus::NotFound => "offline",
    }
}

// ───────────────────────────────────────────────────────────────────
// 解析辅助
// ───────────────────────────────────────────────────────────────────

fn parse_strict_json(s: &str) -> Result<serde_json::Value, ()> {
    serde_json::from_str(s).map_err(|_| ())
}

fn parse_relaxed_json(s: &str) -> Result<serde_json::Value, ()> {
    let cleaned = strip_json_comments_and_trailing_commas(s);
    serde_json::from_str(&cleaned).map_err(|_| ())
}

/// 极简 JSONC 预处理：剥离 `//` 行注释、`/* */` 块注释、尾逗号。
/// 注意：不处理字符串内 // 的极端情况，对常见 settings.json / mcp.json 够用。
fn strip_json_comments_and_trailing_commas(s: &str) -> String {
    // 第一遍：去块注释
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut in_str = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            out.push(b as char);
            if escape { escape = false; } else if b == b'\\' { escape = true; } else if b == b'"' { in_str = false; }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_str = true;
            out.push('"');
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
                continue;
            }
            if bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') { i += 1; }
                i += 2.min(bytes.len() - i);
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    // 第二遍：去尾逗号 ",}" / ",]"
    let mut result = String::with_capacity(out.len());
    let chars: Vec<char> = out.chars().collect();
    let mut idx = 0;
    while idx < chars.len() {
        let c = chars[idx];
        if c == ',' {
            let mut j = idx + 1;
            while j < chars.len() && chars[j].is_whitespace() { j += 1; }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                idx += 1;
                continue;
            }
        }
        result.push(c);
        idx += 1;
    }
    result
}

fn has_required_key_json(v: &serde_json::Value, key: Option<&str>) -> bool {
    let Some(key) = key else { return true; };
    json_get_dot_path(v, key).is_some()
}

fn json_get_dot_path<'a>(v: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut cur = v;
    for seg in path.split('.') {
        cur = cur.get(seg)?;
    }
    Some(cur)
}

fn check_dotenv(s: &str, required: Option<&str>) -> bool {
    let lines = s.lines().filter_map(|l| {
        let t = l.trim();
        if t.is_empty() || t.starts_with('#') { return None; }
        let eq = t.find('=')?;
        Some((t[..eq].trim().to_string(), t[eq + 1..].trim().to_string()))
    });
    if let Some(key) = required {
        lines.into_iter().any(|(k, v)| k == key && !v.is_empty())
    } else {
        lines.into_iter().next().is_some()
    }
}

fn check_toml_lookalike(s: &str, required: Option<&str>) -> bool {
    let key = match required {
        None => {
            return s.lines().any(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#') && t.contains('=')
            });
        }
        Some(k) => k,
    };
    let segments: Vec<&str> = key.split('.').collect();
    if segments.len() == 1 {
        return s.lines().any(|l| {
            let t = l.trim();
            if t.starts_with('#') { return false; }
            let prefix = format!("{}", segments[0]);
            (t.starts_with(&format!("{} ", prefix)) || t.starts_with(&format!("{}=", prefix)))
                && t.contains('=')
        });
    }
    let section_marker = format!("[{}]", segments[..segments.len() - 1].join("."));
    let key_name = *segments.last().unwrap();
    let mut in_section = false;
    for line in s.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            in_section = t == section_marker;
            continue;
        }
        if in_section && t.contains('=') {
            if t.starts_with(&format!("{} ", key_name)) || t.starts_with(&format!("{}=", key_name))
            {
                return true;
            }
        }
    }
    false
}

fn check_yaml_lookalike(s: &str, required: Option<&str>) -> bool {
    let key = match required {
        None => {
            return s.lines().any(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#') && t.contains(':')
            });
        }
        Some(k) => k,
    };
    let top = key.split('.').next().unwrap_or(key);
    s.lines().any(|l| {
        let t = l.trim_start();
        if t.starts_with('#') { return false; }
        t.starts_with(&format!("{}:", top))
    })
}

// ───────────────────────────────────────────────────────────────────
// 测试
// ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relaxed_json_strips_comments_and_trailing_commas() {
        let v = parse_relaxed_json(r#"{
            // top
            "a": 1,
            "b": [1, 2,],
        }"#);
        assert!(v.is_ok());
    }

    #[test]
    fn dotenv_required_key_present() {
        let s = "# comment\nFOO=bar\nGEMINI_API_KEY=xxx\n";
        assert!(check_dotenv(s, Some("GEMINI_API_KEY")));
        assert!(!check_dotenv(s, Some("MISSING")));
    }

    #[test]
    fn dotenv_required_key_empty_value_rejected() {
        let s = "GEMINI_API_KEY=\n";
        assert!(!check_dotenv(s, Some("GEMINI_API_KEY")));
    }

    #[test]
    fn toml_dotpath() {
        let s = "[models.providers]\nfoo = \"bar\"\n";
        assert!(check_toml_lookalike(s, Some("models.providers.foo")));
    }

    #[test]
    fn empty_file_is_broken_not_active() {
        let tmp = std::env::temp_dir().join("__detect_empty.json");
        std::fs::write(&tmp, "").unwrap();
        let r = detect_config(&[ConfigCandidate::new(tmp.clone(), ConfigFormat::Json)]);
        assert_eq!(r.status, DetectionStatus::ConfigBroken);
        let _ = std::fs::remove_file(&tmp);
    }
}
