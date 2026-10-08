use crate::campaign::Campaign;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct CampaignMetrics {
    pub id: usize,
    pub symbol: String,
    pub start: chrono::NaiveDate,
    pub end: Option<chrono::NaiveDate>,
    pub stock_pnl: Decimal,
    pub option_pnl: Decimal,
    pub total_pnl: Decimal,
    pub peak_net_capital: Decimal,
    pub return_on_capital: Option<Decimal>,
    pub option_transaction_count: usize,
    pub status: String,
}

fn stock_pnl(c: &Campaign) -> Decimal {
    c.stock_transactions.iter().map(|t| t.amount + t.fees).sum()
}
fn option_pnl(c: &Campaign) -> Decimal {
    c.option_transactions
        .iter()
        .map(|t| t.amount + t.fees)
        .sum()
}

fn peak_net_capital(c: &Campaign) -> Decimal {
    let mut cash = Decimal::ZERO;
    let mut min_cash = Decimal::ZERO;
    let mut events: Vec<_> = c
        .stock_transactions
        .iter()
        .map(|t| (t.date, t.amount + t.fees))
        .chain(
            c.option_transactions
                .iter()
                .map(|t| (t.date, t.amount + t.fees)),
        )
        .collect();
    events.sort_by_key(|x| x.0);
    for (_, cf) in events {
        cash += cf;
        if cash < min_cash {
            min_cash = cash;
        }
    }
    -min_cash
}

pub fn calculate_campaign_metrics(campaigns: &[Campaign]) -> Vec<CampaignMetrics> {
    let mut cm: Vec<CampaignMetrics> = campaigns
        .iter()
        .map(|c| {
            let sp = stock_pnl(c);
            let op = option_pnl(c);
            let total = sp + op;
            let peak = peak_net_capital(c);
            CampaignMetrics {
                id: c.id,
                symbol: c.symbol.clone(),
                start: c.start,
                end: c.end,
                stock_pnl: sp,
                option_pnl: op,
                total_pnl: total,
                peak_net_capital: peak,
                return_on_capital: (peak.is_zero()).then_some(Decimal::ZERO).or_else(|| {
                    if peak.is_sign_positive() {
                        Some(total / peak)
                    } else {
                        None
                    }
                }),
                option_transaction_count: c.option_transactions.len(),
                status: if c.end.is_some() {
                    "closed".into()
                } else {
                    "open".into()
                },
            }
        })
        .collect();
    // Sort campaigns metrics vector by decreasing start date
    cm.sort_by_key(|c| c.start);
    cm
}

pub fn summarize(m: &[CampaignMetrics]) {
    let c: Vec<_> = m.iter().filter(|x| x.status == "closed").collect();
    let win_total: Decimal = c
        .iter()
        .filter(|x| x.total_pnl > Decimal::ZERO)
        .map(|x| x.total_pnl)
        .sum();
    let wins = c.iter().filter(|x| x.total_pnl > Decimal::ZERO).count();
    let loss_total: Decimal = c
        .iter()
        .filter(|x| x.total_pnl < Decimal::ZERO)
        .map(|x| x.total_pnl)
        .sum();
    let losses = c.iter().filter(|x| x.total_pnl < Decimal::ZERO).count();
    let pnl: Decimal = c.iter().map(|x| x.total_pnl).sum();
    let gp: Decimal = c
        .iter()
        .filter(|x| x.total_pnl > Decimal::ZERO)
        .map(|x| x.total_pnl)
        .sum();
    let gl: Decimal = c
        .iter()
        .filter(|x| x.total_pnl < Decimal::ZERO)
        .map(|x| x.total_pnl.abs())
        .sum();

    // Print as a table with columns for id, symbol, start, end, stock_pnl, option_pnl, total_pnl, peak_net_capital, return_on_capital, option_transaction_count, and status
    println!();
    println!(
        "{:<6} {:<10} {:<12} {:<12} {:<15} {:<15} {:<15} {:<20} {:<20} {:<10} {:<10}",
        "ID",
        "Symbol",
        "Start",
        "End",
        "Stock P&L",
        "Option P&L",
        "Total P&L",
        "Peak Net Capital",
        "Return on Capital",
        "Option Txns",
        "Status"
    );
    println!("{}", "-".repeat(150));
    for x in m {
        println!(
            "{:<6} {:<10} {:<12} {:<12} ${:<15.2} ${:<15.2} ${:<15.2} ${:<20.2} {:<20} {:<10} {:<10}",
            x.id,
            x.symbol,
            x.start,
            x.end.map_or("N/A".to_string(), |d| d.to_string()),
            x.stock_pnl,
            x.option_pnl,
            x.total_pnl,
            x.peak_net_capital,
            format!(
                "{:.2}%",
                x.return_on_capital
                    .map_or(Decimal::ZERO, |r| r * Decimal::from(100))
            ),
            x.option_transaction_count,
            x.status
        );
    }

    // Calculate campaign profit per day for each campaign and print the average
    let total_profit_per_day: Decimal = c
        .iter()
        .filter_map(|x| {
            if let Some(end) = x.end {
                let days = (end - x.start).num_days();
                if days > 0 {
                    Some(x.total_pnl / Decimal::from(days))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .sum::<Decimal>();

    println!();
    println!(
        "{:<6} {:<10} {:<10} {:<10} {:<10}  {:<10}  {:<15}  {:<15}  {:<15} {:<10} {:<20} {:<10} {:<10}",
        "Total",
        "Win",
        "Loss",
        "WinRate",
        "AvgWinPL",
        "AvgLossPL",
        "StockPL",
        "OptionPL",
        "CampPL",
        "ProfitFac",
        "NetCapital",
        "AvgROC",
        "AvgProfitPDay"
    );
    println!("{}", "-".repeat(170));
    println!(
        "{:<6} {:<10} {:<10} {:<10} ${:<10.2} ${:<10.2} ${:<15.2} ${:<15.2} ${:<15.2} {:<10.2} ${:<20.2} {:<10} ${:<10.2}",
        c.len(),
        wins,
        losses,
        if !c.is_empty() {
            format!(
                "{:.2}%",
                Decimal::from(wins as i64) / Decimal::from(c.len() as i64) * Decimal::from(100)
            )
        } else {
            format!("{:.2}%", Decimal::ZERO)
        },
        if wins > 0 {
            win_total / Decimal::from(wins as i64)
        } else {
            Decimal::ZERO
        },
        if losses > 0 {
            loss_total / Decimal::from(losses as i64)
        } else {
            Decimal::ZERO
        },
        c.iter().map(|x| x.stock_pnl).sum::<Decimal>(),
        c.iter().map(|x| x.option_pnl).sum::<Decimal>(),
        pnl,
        if gl > Decimal::ZERO {
            gp / gl
        } else {
            Decimal::ZERO
        },
        c.iter().map(|x| x.peak_net_capital).sum::<Decimal>(),
        if (c.iter().map(|x| x.peak_net_capital).sum::<Decimal>()) > Decimal::ZERO {
            let temp = c
                .iter()
                .filter_map(|x| x.return_on_capital)
                .sum::<Decimal>()
                / Decimal::from(c.len() as i64)
                * Decimal::from(100);
            format!("{:.2}%", temp)
        } else {
            format!("{:.2}%", Decimal::ZERO)
        },
        if !c.is_empty() {
            total_profit_per_day / Decimal::from(c.len() as i64)
        } else {
            Decimal::ZERO
        }
    );
}
