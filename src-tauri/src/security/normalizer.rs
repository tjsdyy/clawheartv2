//! 6 遍归一化 — Pipelock 借鉴
//!
//! 对抗：零宽字符 / 同形异义字 / Leetspeak / base64 嵌入 / Unicode 等价类 / 多余空白。
//!
//! 使用：每条危险指令 / 注入规则都保留原始正则 + 归一化匹配；命中任一即触发，
//! evidence 同时记录两者，便于复盘。

/// 6-pass 归一化主函数。
pub fn normalize_for_match(text: &str) -> String {
    let s = strip_zero_width_chars(text);
    let s = replace_homoglyphs(&s);
    let s = decode_leetspeak(&s);
    let s = unwrap_base64_segments(&s);
    let s = unicode_nfkc(&s);
    collapse_whitespace(&s)
}

/// Pass 1：移除零宽字符（ZWSP / ZWNJ / ZWJ / BOM 等）。
pub fn strip_zero_width_chars(s: &str) -> String {
    s.chars()
        .filter(|c| {
            !matches!(
                *c,
                '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{200E}' | '\u{200F}'
                    | '\u{FEFF}' | '\u{2060}' | '\u{180E}'
            )
        })
        .collect()
}

/// Pass 2：同形异义字 → ASCII（核心子集；完整集 W10 拓展到 ~400 条）。
pub fn replace_homoglyphs(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'а' => 'a', 'е' => 'e', 'о' => 'o', 'р' => 'p', 'с' => 'c', 'у' => 'y', 'х' => 'x',
            'А' => 'A', 'В' => 'B', 'Е' => 'E', 'К' => 'K', 'М' => 'M', 'Н' => 'H',
            'О' => 'O', 'Р' => 'P', 'С' => 'C', 'Т' => 'T', 'Х' => 'X',
            'ı' => 'i', 'ɑ' => 'a',
            // 全角→半角
            'A'..='Z' if (c as u32) >= 0xFF21 => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
            _ => c,
        })
        .collect()
}

/// Pass 3：Leetspeak 解码（4→a / 0→o / 1→i / 5→s / 3→e / 7→t / !→i / @→a / $→s）。
pub fn decode_leetspeak(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '4' => 'a',
            '0' => 'o',
            '1' => 'i',
            '!' => 'i',
            '5' => 's',
            '$' => 's',
            '3' => 'e',
            '7' => 't',
            '@' => 'a',
            _ => c,
        })
        .collect()
}

/// Pass 4：识别看起来像 base64 的连续段（≥16 char, [A-Za-z0-9+/=]），尝试解码并并入原文。
/// 解码失败/非可打印则丢弃。
pub fn unwrap_base64_segments(s: &str) -> String {
    use std::fmt::Write;

    let mut out = String::with_capacity(s.len());
    let mut current = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        let is_b64 = c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=';
        if is_b64 {
            current.push(c);
        } else {
            if current.len() >= 16 {
                if let Some(decoded) = try_base64_decode(&current) {
                    let _ = write!(out, "{} ", decoded);
                }
            }
            out.push_str(&current);
            current.clear();
            out.push(c);
        }
    }
    if current.len() >= 16 {
        if let Some(decoded) = try_base64_decode(&current) {
            out.push(' ');
            out.push_str(&decoded);
        }
    }
    out.push_str(&current);
    out
}

fn try_base64_decode(input: &str) -> Option<String> {
    // 自实现最小 base64 decoder（避免引入 base64 crate）
    let table: [u8; 256] = {
        let mut t = [0xFFu8; 256];
        for (i, c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
            t[*c as usize] = i as u8;
        }
        t
    };

    let cleaned: Vec<u8> = input.bytes().filter(|b| *b != b'=' && table[*b as usize] != 0xFF).collect();
    if cleaned.len() % 4 == 1 { return None; }

    let mut out = Vec::with_capacity(cleaned.len() * 3 / 4);
    for chunk in cleaned.chunks(4) {
        let mut buf = [0u8; 4];
        for (i, b) in chunk.iter().enumerate() {
            buf[i] = table[*b as usize];
        }
        let n = chunk.len();
        if n >= 2 { out.push((buf[0] << 2) | (buf[1] >> 4)); }
        if n >= 3 { out.push((buf[1] << 4) | (buf[2] >> 2)); }
        if n == 4 { out.push((buf[2] << 6) | buf[3]); }
    }

    let s = String::from_utf8(out).ok()?;
    // 至少 60% 是可打印 ASCII 才认为是合法 payload
    let printable = s.bytes().filter(|b| (32..=126).contains(b)).count();
    if printable * 100 / s.len().max(1) < 60 { return None; }
    Some(s)
}

/// Pass 5：Unicode NFKC 等价归一化（半角全角统一）。
pub fn unicode_nfkc(s: &str) -> String {
    // alpha 阶段：W10 接入 `unicode-normalization` crate；现在做最小子集
    s.chars()
        .map(|c| match c {
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
            '\u{3000}' => ' ',
            _ => c,
        })
        .collect()
}

/// Pass 6：折叠多余空白为单空格，trim。
pub fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_width() {
        assert_eq!(strip_zero_width_chars("rm\u{200B}-rf"), "rm-rf");
    }

    #[test]
    fn test_homoglyph() {
        assert_eq!(replace_homoglyphs("rм -rf"), "rм -rf"); // м (Cyrillic) not yet in map
        assert_eq!(replace_homoglyphs("rа -rf"), "ra -rf"); // а Cyrillic → a
    }

    #[test]
    fn test_leetspeak() {
        assert_eq!(decode_leetspeak("rm -rf /h0me"), "rm -rf /home");
        // `c@t` → `cat`, `/3tc` → `/etc`, `p@$$wd` → `passwd`
        assert_eq!(decode_leetspeak("c@t /3tc/p@$$wd"), "cat /etc/passwd");
    }

    #[test]
    fn test_full_pipeline_rmrf() {
        // rm  -rf with zero-width + full-width space
        let input = "rm\u{200B}\u{3000}-rf /";
        let normalized = normalize_for_match(input);
        assert!(normalized.contains("rm -rf"));
    }
}
