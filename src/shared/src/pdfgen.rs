//! Minimal single-page text PDF writer, used only for demo/sample documents.

fn escape(line: &str) -> String {
    line.chars()
        .filter(|c| c.is_ascii() && !c.is_ascii_control())
        .flat_map(|c| match c {
            '(' | ')' | '\\' => vec!['\\', c],
            _ => vec![c],
        })
        .collect()
}

pub fn simple_pdf(lines: &[String]) -> Vec<u8> {
    let mut content = String::from("BT\n/F1 12 Tf\n72 740 Td\n16 TL\n");
    for line in lines {
        content.push_str(&format!("({}) Tj\nT*\n", escape(line)));
    }
    content.push_str("ET\n");

    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
         /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>"
            .to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_string(),
        format!("<< /Length {} >>\nstream\n{}endstream", content.len(), content),
    ];

    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, obj).as_bytes());
    }
    let xref_at = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in offsets {
        out.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_at
        )
        .as_bytes(),
    );
    out
}

fn money(v: i64) -> String {
    let digits = v.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if v < 0 { format!("(${out})") } else { format!("${out}") }
}

const DEMO_NOTE: &str = "DEMO DOCUMENT - fictional data generated for LoanDesk";

pub fn financial_statement(
    business: &str,
    revenue: i64,
    net_income: i64,
    total_debt: i64,
) -> Vec<u8> {
    simple_pdf(&[
        business.to_string(),
        "Annual Financial Statement - Fiscal Year Ended December 31".to_string(),
        DEMO_NOTE.to_string(),
        String::new(),
        "INCOME STATEMENT".to_string(),
        format!("Total revenue: {}", money(revenue)),
        format!("Operating costs and expenses: {}", money(revenue - net_income)),
        format!("Net income: {}", money(net_income)),
        String::new(),
        "BALANCE SHEET HIGHLIGHTS".to_string(),
        format!("Total debt: {}", money(total_debt)),
    ])
}

pub fn tax_return(
    business: &str,
    ein: &str,
    revenue: i64,
    net_income: i64,
    total_debt: i64,
) -> Vec<u8> {
    simple_pdf(&[
        "U.S. Business Income Tax Return (summary)".to_string(),
        DEMO_NOTE.to_string(),
        String::new(),
        format!("Name of business: {business}"),
        format!("Employer identification number: {ein}"),
        String::new(),
        format!("Gross receipts or sales: {}", money(revenue)),
        format!("Net income (loss) per books: {}", money(net_income)),
        format!("Total debt outstanding: {}", money(total_debt)),
    ])
}

pub fn bank_statement(business: &str, opening: i64, closing: i64) -> Vec<u8> {
    simple_pdf(&[
        "First Meridian Bank - Business Checking Statement".to_string(),
        DEMO_NOTE.to_string(),
        String::new(),
        format!("Account holder: {business}"),
        "Account number ending: 4471".to_string(),
        String::new(),
        format!("Beginning balance: {}", money(opening)),
        format!("Deposits and credits: {}", money(closing - opening + 212_400)),
        "Withdrawals and debits: $212,400".to_string(),
        format!("Ending balance: {}", money(closing)),
    ])
}

/// Files written by `loandesk-api gen-samples`, meant to be uploaded by hand
/// to the seeded "Cedar Ridge Veterinary Clinic" application.
pub fn sample_documents() -> Vec<(&'static str, Vec<u8>)> {
    let name = "Cedar Ridge Veterinary Clinic PC";
    vec![
        (
            "cedar-ridge-financial-statement.pdf",
            financial_statement(name, 2_850_000, 385_000, 420_000),
        ),
        ("cedar-ridge-tax-return.pdf", tax_return(name, "18-5521906", 2_850_000, 385_000, 420_000)),
        ("cedar-ridge-bank-statement.pdf", bank_statement(name, 148_200, 171_950)),
    ]
}
