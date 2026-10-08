use crate::transaction::Transaction;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use csv::StringRecord;
use rust_decimal::Decimal;
use std::fs::File;

fn clean(s: Option<&str>) -> &str {
    s.unwrap_or("").trim().trim_matches('"')
}

fn decimal(s: &str) -> Result<Decimal> {
    let s = s.trim().trim_matches('"').replace(['$', ','], "");
    if s.is_empty() {
        return Ok(Decimal::ZERO);
    }
    Ok(s.parse()?)
}

fn date(s: &str) -> Result<NaiveDate> {
    for f in ["%m/%d/%Y", "%Y-%m-%d", "%m/%d/%y"] {
        if let Ok(d) = NaiveDate::parse_from_str(s.split_whitespace().next().unwrap_or(""), f) {
            return Ok(d);
        }
    }
    anyhow::bail!("Unsupported date: {s}")
}

fn field(h: &StringRecord, r: &StringRecord, name: &str) -> Result<String> {
    let i = h
        .iter()
        .position(|x| x.trim().trim_matches('"') == name)
        .with_context(|| format!("Missing CSV column: {name}"))?;
    Ok(clean(r.get(i)).to_string())
}

pub fn read_transactions(path: &str) -> Result<Vec<Transaction>> {
    let file = File::open(path).with_context(|| format!("Cannot open {path}"))?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(file);
    let headers = rdr.headers()?.clone();
    let mut out = Vec::new();
    for (n, rec) in rdr.records().enumerate() {
        let row = rec.with_context(|| format!("CSV row {}", n + 2))?;
        out.push(Transaction {
            date: date(&field(&headers, &row, "Date")?)?,
            action: field(&headers, &row, "Action")?,
            symbol: field(&headers, &row, "Symbol")?.to_ascii_uppercase(),
            description: field(&headers, &row, "Description")?,
            quantity: decimal(&field(&headers, &row, "Quantity")?)?,
            price: decimal(&field(&headers, &row, "Price")?)?,
            fees: decimal(&field(&headers, &row, "Fees & Comm")?)?,
            amount: decimal(&field(&headers, &row, "Amount")?)?,
        });
    }
    // Sort transactions by date
    out.sort_by_key(|t| t.date);
    Ok(out)
}
