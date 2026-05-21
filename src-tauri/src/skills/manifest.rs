//! 极简 SKILL.md frontmatter 解析器。
//!
//! 只关心首部 `---\n...\n---` 之间的 yaml-ish 内容；
//! 我们仅取常用字段（name / description / version / category / examples）。
//! 完整 yaml 解析（多行字符串、锚点等）不在此 alpha 范围内。

#[derive(Debug, Clone, Default)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub category: Option<String>,
}

/// 提取 markdown 中以 `---` 包围的 frontmatter 并解析为 key/value。
pub fn parse(text: &str) -> SkillFrontmatter {
    let mut fm = SkillFrontmatter::default();
    let trimmed = text.trim_start();
    let Some(rest) = trimmed.strip_prefix("---") else { return fm };
    let rest = rest.strip_prefix('\n').unwrap_or(rest);

    let Some(end) = rest.find("\n---") else { return fm };
    let body = &rest[..end];

    for raw in body.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once(':') else { continue };
        let key = k.trim();
        let value = v.trim().trim_matches(|c| c == '"' || c == '\'');
        if value.is_empty() {
            continue;
        }
        match key {
            "name" => fm.name = Some(value.to_string()),
            "description" => fm.description = Some(value.to_string()),
            "version" => fm.version = Some(value.to_string()),
            "category" => fm.category = Some(value.to_string()),
            _ => {}
        }
    }

    fm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_frontmatter() {
        let md = "---\nname: my-skill\ndescription: \"A nice skill\"\nversion: 1.0.0\n---\n\nbody";
        let fm = parse(md);
        assert_eq!(fm.name.as_deref(), Some("my-skill"));
        assert_eq!(fm.description.as_deref(), Some("A nice skill"));
        assert_eq!(fm.version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn returns_default_when_no_frontmatter() {
        let fm = parse("# Just a heading\n\nbody only");
        assert!(fm.name.is_none() && fm.version.is_none());
    }

    #[test]
    fn ignores_comments_and_empty_lines() {
        let md = "---\n# this is a comment\n\nname: x\n---\n";
        assert_eq!(parse(md).name.as_deref(), Some("x"));
    }
}
