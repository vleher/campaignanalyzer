use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub date: NaiveDate,
    pub action: String,
    pub symbol: String,
    pub description: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub fees: Decimal,
    pub amount: Decimal,
}

impl Transaction {
    pub fn is_stock(&self) -> bool {
        !self.symbol.contains(' ')
    }
    pub fn is_option(&self) -> bool {
        !self.is_stock()
    }
}
