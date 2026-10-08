use anyhow::Result;
use campaignanalyzer::{aggregation, campaign, metrics, parser};
use chrono::NaiveDate;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    // Stock symbol
    #[arg(short, long, default_value_t = String::from("ALLSTOCKS"))]
    stock: String,
    // Start Date
    #[arg(short, long, default_value_t = String::from("2020-01-01"))]
    date: String,
}

fn main() -> Result<()> {
    // read env arg with -s	flag
    let args = Args::parse();
    let start_date = NaiveDate::parse_from_str(&args.date, "%Y-%m-%d")?;

    println!(
        "Starting analysis with stock: {}, date: {}.",
        args.stock, args.date
    );

    let input = "/home/leher/Documents/stocks/brokerage-transactions-all.csv".to_string();
    let transactions = parser::read_transactions(&input)?;
    println!("Loaded {} transactions.", transactions.len());
    // Filter transactions by stock symbol if provided
    let transactions: Vec<_> = transactions
        .into_iter()
        .filter(|t| t.date >= start_date)
        .filter(|t| {
            args.stock.eq_ignore_ascii_case("ALLSTOCKS")
                || t.symbol.split_whitespace().next().unwrap_or_default() == args.stock
        })
        .collect();

    if args.stock != "ALLSTOCKS" {
        println!(
            "Filtered to {} transactions for symbol {} and date {}.",
            transactions.len(),
            args.stock,
            start_date
        );
    }
    // Build and  attach options to campaigns
    let mut campaigns = campaign::build_campaigns(&transactions);
    campaign::attach_options(&mut campaigns, &transactions);
    let m = metrics::calculate_campaign_metrics(&campaigns);
    metrics::summarize(&m);

    let monthly_totals = aggregation::aggregate_monthly_transactions(&transactions);
    aggregation::summarize_totals(&monthly_totals);

    let yearly_totals = aggregation::aggregate_yearly_transactions(&transactions);
    aggregation::summarize_totals(&yearly_totals);

    Ok(())
}
