use std::collections::HashMap;

use chrono::Datelike;
use rust_decimal::Decimal;

use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Totals {
    buy_to_close: Decimal,
    sell_to_open: Decimal,
    buy: Decimal,
    sell: Decimal,
    net_option: Decimal,
    net_stock: Decimal,
    net_trading: Decimal,
    net_investment: Decimal,
    deposit_total: Decimal,
    withdrawal_total: Decimal,
}

#[derive(Debug, Clone)]
pub struct PositionTotals {
    option_total: Decimal,
    stock_total: Decimal,
    all_total: Decimal,
}

impl Default for Totals {
    fn default() -> Self {
        Self::new()
    }
}

impl Totals {
    pub fn new() -> Self {
        Totals {
            buy_to_close: Decimal::from(0),
            sell_to_open: Decimal::from(0),
            buy: Decimal::from(0),
            sell: Decimal::from(0),
            net_option: Decimal::from(0),
            net_stock: Decimal::from(0),
            net_trading: Decimal::from(0),
            net_investment: Decimal::from(0),
            withdrawal_total: Decimal::from(0),
            deposit_total: Decimal::from(0),
        }
    }
}

impl Default for PositionTotals {
    fn default() -> Self {
        Self::new()
    }
}

impl PositionTotals {
    pub fn new() -> Self {
        PositionTotals {
            option_total: Decimal::from(0),
            stock_total: Decimal::from(0),
            all_total: Decimal::from(0),
        }
    }
}

pub fn aggregate_monthly_transactions(transactions: &[Transaction]) -> HashMap<(i32, u32), Totals> {
    aggregate_transactions(transactions, true)
}

pub fn aggregate_yearly_transactions(transactions: &[Transaction]) -> HashMap<(i32, u32), Totals> {
    aggregate_transactions(transactions, false)
}

fn aggregate_transactions(
    transactions: &[Transaction],
    monthly: bool,
) -> HashMap<(i32, u32), Totals> {
    let mut totals: HashMap<(i32, u32), Totals> = HashMap::new();
    for transaction in transactions {
        let year = transaction.date.year();
        let mut month = 0;
        if monthly {
            month = transaction.date.month();
        }
        let key = (year, month);
        let entry = totals.entry(key).or_default();
        match transaction.action.as_str() {
            "Buy to Close" => entry.buy_to_close += transaction.amount,
            "Sell to Open" => entry.sell_to_open += transaction.amount,
            "Buy" => entry.buy += transaction.amount,
            "Sell" => entry.sell += transaction.amount,
            "Journal" => {
                if transaction
                    .description
                    .to_lowercase()
                    .contains("transfer funds from")
                {
                    entry.deposit_total += transaction.amount;
                } else if transaction
                    .description
                    .to_lowercase()
                    .contains("transfer funds to")
                {
                    entry.withdrawal_total += transaction.amount;
                }
            }
            _ => {}
        }

        match transaction.action.as_str() {
            "Buy to Close" | "Sell to Open" => entry.net_option += transaction.amount,
            "Buy" | "Sell" => entry.net_stock += transaction.amount,
            _ => {}
        }
        entry.net_trading = entry.net_option + entry.net_stock;
        entry.net_investment = entry.buy.abs() - entry.sell.abs() + entry.sell_to_open.abs()
            - entry.buy_to_close.abs();
    }

    totals
}

pub fn aggregate_position_totals(transactions: &[Transaction]) -> HashMap<String, PositionTotals> {
    let mut position_totals: HashMap<String, PositionTotals> = HashMap::new();
    for transaction in transactions {
        let position_key = transaction.symbol.clone();
        let entry = position_totals.entry(position_key).or_default();
        match transaction.action.as_str() {
            "Buy to Close" | "Sell to Open" => entry.option_total += transaction.amount,
            "Buy" | "Sell" => entry.stock_total += transaction.amount,
            _ => {}
        }
        entry.all_total = entry.option_total + entry.stock_total;
    }
    position_totals
}

pub fn summarize_totals(totals: &HashMap<(i32, u32), Totals>) {
    // Sort hashmap by year and month
    let mut totals: Vec<(&(i32, u32), &Totals)> = totals.iter().collect();
    totals.sort_by_key(|&(k, _)| k);
    println!();
    // Print the totals as a table with columns for year, month, buy to close, sell to open, buy, sell, option total, stock total, all total, deposit total, and withdrawal total
    println!(
        "{:<6} {:<6} {:<15} {:<15} {:<15} {:<15} {:<15} {:<15} {:<15} {:<15} {:<15} {:<15}",
        "Year",
        "Month",
        "OptionPremium",
        "OptionBuyback",
        "StockBuy",
        "StockSell",
        "StockNet",
        "OptionNet",
        "NetTrading",
        "NetInvestment",
        "NetCashInflow",
        "NetCashOutflow"
    );
    println!("{}", "-".repeat(172));

    for ((year, month), totals) in &totals {
        println!(
            "{:<6} {:<6} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2}",
            year,
            month,
            totals.sell_to_open,
            totals.buy_to_close,
            totals.buy,
            totals.sell,
            totals.net_stock,
            totals.net_option,
            totals.net_trading,
			totals.net_investment,
            totals.deposit_total,
            totals.withdrawal_total
        );
    }

    // Sum each column and print the totals
    let mut total_buy_to_close = Decimal::from(0);
    let mut total_sell_to_open = Decimal::from(0);
    let mut total_buy = Decimal::from(0);
    let mut total_sell = Decimal::from(0);
    let mut total_option_total = Decimal::from(0);
    let mut total_stock_total = Decimal::from(0);
    let mut total_all_total = Decimal::from(0);
    let mut total_net_investment = Decimal::from(0);
    let mut total_deposit_total = Decimal::from(0);
    let mut total_withdrawal_total = Decimal::from(0);

    for (_, totals) in &totals {
        total_buy_to_close += totals.buy_to_close;
        total_sell_to_open += totals.sell_to_open;
        total_buy += totals.buy;
        total_sell += totals.sell;
        total_option_total += totals.net_option;
        total_stock_total += totals.net_stock;
        total_all_total += totals.net_trading;
        total_net_investment += totals.net_investment;
        total_deposit_total += totals.deposit_total;
        total_withdrawal_total += totals.withdrawal_total;
    }
    println!();
    println!(
        "{:<6} {:<6} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2}",
        "Total",
        "",
        total_sell_to_open,
        total_buy_to_close,
        total_buy,
        total_sell,
        total_stock_total,
        total_option_total,
        total_all_total,
		total_net_investment,
        total_deposit_total,
        total_withdrawal_total
    );
}
