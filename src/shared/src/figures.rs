//! Pulls revenue, net income and total debt out of document text.
//!
//! This is deliberately simple label matching: a line has to start with a
//! known label ("Total revenue", "Net income", "Total debt", ...) followed by
//! an amount. Anything it does not recognise is left empty rather than guessed.

use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Figures {
    pub revenue: Option<i64>,
    pub net_income: Option<i64>,
    pub total_debt: Option<i64>,
}

const AMOUNT: &str = r"(\(?-?\s*\$?\s*\d[\d,]*(?:\.\d+)?\)?)";
const GAP: &str = r"[^\d(\-\n]*";

fn regexes() -> &'static [Regex; 3] {
    static RE: OnceLock<[Regex; 3]> = OnceLock::new();
    RE.get_or_init(|| {
        let build = |label: &str| {
            Regex::new(&format!(r"(?im)^\s*{label}{GAP}{AMOUNT}")).expect("valid regex")
        };
        [
            build(r"(?:total\s+|gross\s+|net\s+)?(?:revenues?|sales|receipts(?:\s+or\s+sales)?)\b"),
            build(r"net\s+(?:income|profit|earnings)\b(?:\s*\(loss\))?"),
            build(r"total\s+(?:outstanding\s+)?(?:debt|borrowings|loans)\b"),
        ]
    })
}

fn parse_amount(raw: &str) -> Option<i64> {
    let negative = raw.contains('(') || raw.contains('-');
    let cleaned: String = raw.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    let value: f64 = cleaned.parse().ok()?;
    if !value.is_finite() || value > 1e15 {
        return None;
    }
    let whole = value.round() as i64;
    Some(if negative { -whole } else { whole })
}

fn find(re: &Regex, text: &str) -> Option<i64> {
    re.captures(text).and_then(|c| c.get(1)).and_then(|m| parse_amount(m.as_str()))
}

pub fn parse_figures(text: &str) -> Figures {
    let [revenue, net_income, total_debt] = regexes();
    Figures {
        revenue: find(revenue, text),
        net_income: find(net_income, text),
        total_debt: find(total_debt, text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn financial_statement() {
        let f = parse_figures(
            "Acme LLC\nTotal revenue: $4,200,000\nOperating expenses: $1,270,000\n\
             Net income: $620,000.40\nTotal debt: $900,000\n",
        );
        assert_eq!(f.revenue, Some(4_200_000));
        assert_eq!(f.net_income, Some(620_000));
        assert_eq!(f.total_debt, Some(900_000));
    }

    #[test]
    fn tax_return_labels_and_losses() {
        let f = parse_figures(
            "  Gross receipts or sales  2,100,000\nNet income (loss) per books: ($45,000)\n\
             Total debt outstanding: $800,000",
        );
        assert_eq!(f.revenue, Some(2_100_000));
        assert_eq!(f.net_income, Some(-45_000));
        assert_eq!(f.total_debt, Some(800_000));
    }

    #[test]
    fn unrelated_text_gives_nothing() {
        let f = parse_figures(
            "Beginning balance: $10,000\nDeferred revenue: $5\nEnding balance: $12,000",
        );
        assert_eq!(f, Figures::default());
    }
}
