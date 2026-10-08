use campaignanalyzer::option_symbol::{OptionType, parse_option_symbol};
use chrono::NaiveDate;

#[test]
fn parses_schwab_call() {
    let c = parse_option_symbol("NVDA 09/18/2026 180 C").unwrap();
    assert_eq!(c.underlying, "NVDA");
    assert_eq!(c.expiration, NaiveDate::from_ymd_opt(2026, 9, 18).unwrap());
    assert_eq!(c.option_type, OptionType::Call);
}
