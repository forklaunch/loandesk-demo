//! Simple risk summary.
//!
//! These formulas and thresholds are placeholders so the screen has something
//! sensible to show. They are NOT a credit policy. Replace them with the
//! team's real policy before anyone relies on the flag.
//!
//! - Total debt        = existing debt (from documents) + requested amount
//! - Annual debt service = level annual payment on total debt at
//!   `ASSUMED_RATE` over `ASSUMED_TERM_YEARS`
//! - DSCR              = net income / annual debt service
//! - Leverage          = total debt / annual revenue
//! - Flag: high   if DSCR < 1.10 or leverage > 1.00
//!         low    if DSCR >= 1.50 and leverage <= 0.50
//!         medium otherwise

use serde::Serialize;

pub const ASSUMED_RATE: f64 = 0.08;
pub const ASSUMED_TERM_YEARS: i32 = 10;

#[derive(Serialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RiskFlag {
    Low,
    Medium,
    High,
}

#[derive(Serialize, Debug)]
pub struct RiskSummary {
    pub dscr: Option<f64>,
    pub leverage: Option<f64>,
    pub annual_debt_service: Option<f64>,
    /// None when the documents have not yet given us all three figures.
    pub flag: Option<RiskFlag>,
    pub missing: Vec<&'static str>,
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub fn annual_debt_service(total_debt: f64) -> f64 {
    let r = ASSUMED_RATE;
    total_debt * r / (1.0 - (1.0 + r).powi(-ASSUMED_TERM_YEARS))
}

pub fn assess(
    requested_amount: i64,
    revenue: Option<i64>,
    net_income: Option<i64>,
    total_debt: Option<i64>,
) -> RiskSummary {
    let mut missing = Vec::new();
    if revenue.is_none() {
        missing.push("revenue");
    }
    if net_income.is_none() {
        missing.push("net income");
    }
    if total_debt.is_none() {
        missing.push("total debt");
    }

    let debt = total_debt.map(|d| (d.max(0) + requested_amount.max(0)) as f64);
    let service = debt.map(annual_debt_service);

    let dscr = match (net_income, service) {
        (Some(ni), Some(s)) if s > 0.0 => Some(ni as f64 / s),
        _ => None,
    };
    let leverage = match (debt, revenue) {
        (Some(d), Some(rev)) if rev > 0 => Some(d / rev as f64),
        _ => None,
    };

    let flag = if !missing.is_empty() {
        None
    } else {
        match (dscr, leverage) {
            (Some(d), Some(l)) => Some(if d < 1.10 || l > 1.00 {
                RiskFlag::High
            } else if d >= 1.50 && l <= 0.50 {
                RiskFlag::Low
            } else {
                RiskFlag::Medium
            }),
            // All figures present but a ratio is undefined (zero or negative
            // revenue): treat as high rather than pretend it is fine.
            _ => Some(RiskFlag::High),
        }
    };

    RiskSummary {
        dscr: dscr.map(round2),
        leverage: leverage.map(round2),
        annual_debt_service: service.map(|s| s.round()),
        flag,
        missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_medium_high() {
        let low = assess(750_000, Some(4_200_000), Some(620_000), Some(900_000));
        assert_eq!(low.flag, Some(RiskFlag::Low));
        assert_eq!(low.leverage, Some(0.39));

        let med = assess(500_000, Some(2_100_000), Some(300_000), Some(800_000));
        assert_eq!(med.flag, Some(RiskFlag::Medium));

        let high = assess(600_000, Some(1_500_000), Some(90_000), Some(1_200_000));
        assert_eq!(high.flag, Some(RiskFlag::High));
    }

    #[test]
    fn missing_figures_give_no_flag() {
        let r = assess(100_000, Some(1_000_000), None, None);
        assert_eq!(r.flag, None);
        assert_eq!(r.missing, vec!["net income", "total debt"]);
    }

    #[test]
    fn loss_or_zero_revenue_is_high() {
        assert_eq!(
            assess(100_000, Some(1_000_000), Some(-50_000), Some(10_000)).flag,
            Some(RiskFlag::High)
        );
        assert_eq!(
            assess(100_000, Some(0), Some(50_000), Some(10_000)).flag,
            Some(RiskFlag::High)
        );
    }
}
