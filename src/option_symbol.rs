use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone)]
pub struct OptionContract {
    pub underlying: String,
    pub expiration: NaiveDate,
    pub strike: String,
    pub option_type: OptionType,
}

/// Parses common Schwab option symbols such as: NVDA 09/18/2026 180 C
pub fn parse_option_symbol(symbol: &str) -> Option<OptionContract> {
    let p: Vec<&str> = symbol.split_whitespace().collect();
    if p.len() < 4 {
        return None;
    }
    let expiration = NaiveDate::parse_from_str(p[1], "%m/%d/%Y").ok()?;
    let option_type = match p[3].to_ascii_uppercase().as_str() {
        "C" => OptionType::Call,
        "P" => OptionType::Put,
        _ => return None,
    };
    Some(OptionContract {
        underlying: p[0].to_ascii_uppercase(),
        expiration,
        strike: p[2].to_string(),
        option_type,
    })
}
