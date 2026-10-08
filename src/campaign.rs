use crate::option_symbol::parse_option_symbol;
use crate::transaction::Transaction;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct Campaign {
    pub id: usize,
    pub symbol: String,
    pub start: NaiveDate,
    pub end: Option<NaiveDate>,
    pub stock_transactions: Vec<Transaction>,
    pub option_transactions: Vec<Transaction>,
}

pub fn build_campaigns(transactions: &[Transaction]) -> Vec<Campaign> {
    let mut txs = transactions.to_vec();
    txs.sort_by_key(|t| t.date);
    let mut symbols: Vec<String> = txs
        .iter()
        .filter(|t| t.is_stock())
        .map(|t| t.symbol.clone())
        .collect();
    symbols.sort();
    symbols.dedup();
    let mut result = Vec::new();
    let mut id = 1usize;

    for symbol in symbols {
        let stock: Vec<Transaction> = txs
            .iter()
            .filter(|t| t.is_stock() && t.symbol == symbol)
            .cloned()
            .collect();
        let mut balance = Decimal::ZERO;
        let mut current: Option<Campaign> = None;
        for t in stock {
            let was_zero = balance == Decimal::ZERO;
            if t.action == "Buy" {
                balance += t.quantity;
            } else if t.action == "Sell" {
                balance -= t.quantity;
            }
            let now_zero = balance == Decimal::ZERO;

            if was_zero && balance > Decimal::ZERO {
                current = Some(Campaign {
                    id,
                    symbol: symbol.clone(),
                    start: t.date,
                    end: None,
                    stock_transactions: vec![t.clone()],
                    option_transactions: vec![],
                });
                id += 1;
            } else if let Some(c) = current.as_mut() {
                c.stock_transactions.push(t.clone());
                if now_zero {
                    c.end = Some(t.date);
                    result.push(current.take().unwrap());
                }
            }
        }
        if let Some(c) = current {
            result.push(c);
        }
    }
    result.sort_by_key(|c| c.start);
    result
}

/// Conservative first-pass association. Overlapping campaigns are left unassigned.
pub fn attach_options(campaigns: &mut [Campaign], transactions: &[Transaction]) {
    for t in transactions.iter().filter(|t| t.is_option()) {
        let Some(contract) = parse_option_symbol(&t.symbol) else {
            continue;
        };
        let matches: Vec<usize> = campaigns
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.symbol == contract.underlying
                    && t.date >= c.start
                    && c.end.map(|e| t.date <= e).unwrap_or(true)
            })
            .map(|(i, _)| i)
            .collect();
        if matches.len() == 1 {
            campaigns[matches[0]].option_transactions.push(t.clone());
        }
    }
}
