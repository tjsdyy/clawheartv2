//! 预算 — 按 provider × model × period 评估
//!
//! 数据从 `storage::queries::budget` 读出（W3 接入）；
//! 现在用纯内存结构占位，方便管线接进来。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    Daily,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetRule {
    pub id: u32,
    pub provider: String,    // "anthropic" / "openai" / "global"
    pub model: Option<String>,
    pub period: Period,
    pub limit_usd: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum BudgetVerdict {
    Pass {
        used_usd: f64,
        limit_usd: f64,
    },
    Warn {
        used_usd: f64,
        limit_usd: f64,
        pct: f64,
    },
    Block {
        used_usd: f64,
        limit_usd: f64,
        rule_id: u32,
    },
}

pub fn evaluate(rules: &[BudgetRule], used_usd: f64, provider: &str, model: &str, period: Period) -> BudgetVerdict {
    for rule in rules {
        if !rule.enabled { continue; }
        if rule.period != period { continue; }
        if rule.provider != "global" && rule.provider != provider { continue; }
        if let Some(m) = &rule.model {
            if m != model { continue; }
        }
        let pct = (used_usd / rule.limit_usd) * 100.0;
        if used_usd >= rule.limit_usd {
            return BudgetVerdict::Block { used_usd, limit_usd: rule.limit_usd, rule_id: rule.id };
        }
        if pct >= 80.0 {
            return BudgetVerdict::Warn { used_usd, limit_usd: rule.limit_usd, pct };
        }
    }
    BudgetVerdict::Pass { used_usd, limit_usd: f64::INFINITY }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_at_limit() {
        let rules = vec![BudgetRule {
            id: 1, provider: "anthropic".into(), model: None,
            period: Period::Daily, limit_usd: 10.0, enabled: true,
        }];
        let v = evaluate(&rules, 10.5, "anthropic", "claude-opus-4-7", Period::Daily);
        assert!(matches!(v, BudgetVerdict::Block { .. }));
    }
}
