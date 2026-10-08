# Campaign Analyzer

Modular Rust prototype for analyzing stock + covered-call/put activity as campaigns.

## Run

```bash
cargo run -- path/to/brokerage-transactions-all.csv
```

## Modules

- `transaction.rs` — normalized transaction model
- `parser.rs` — Schwab CSV parsing
- `option_symbol.rs` — option contract parsing
- `campaign.rs` — stock campaign construction and option association
- `metrics.rs` — campaign P&L and capital-return metrics
- `main.rs` — CLI

## Important

This is a prototype, not tax/accounting software. Option association is intentionally conservative and date-based. A production version should explicitly model option lifecycles, rolls, assignments/exercises, dividends, FIFO lots, short-put obligations, overlapping campaigns, daily marks, and portfolio-level capital efficiency.
