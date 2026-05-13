use crate::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use ta::indicators::{BollingerBands, MovingAverageConvergenceDivergence, RelativeStrengthIndex};
use ta::Next;
use tracing::{error, info};
use yahoo_finance_api as yahoo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSignal {
    pub ticker: String,
    pub price: f64,
    pub rsi: f64,
    pub macd_hist: f64,
    pub bb_upper: f64,
    pub bb_lower: f64,
    pub signal: String, // "STRONG BUY", "STRONG SELL", or "NEUTRAL"
    pub timestamp: i64,
}

pub async fn start_stock_worker(state: AppState) {
    info!("Starting IDX stock background worker...");
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    // Example ticker list for IDX
    let tickers = vec!["BBCA.JK", "BBRI.JK", "TLKM.JK", "ASII.JK", "GOTO.JK"];

    loop {
        interval.tick().await;
        info!("Running stock analysis for IDX...");

        for ticker in &tickers {
            match fetch_and_analyze_stock(ticker).await {
                Ok(signal_data) => {
                    // Broadcast via SSE as requested
                    if let Err(e) = state
                        .broadcast_sse("stock_signal", json!(signal_data))
                        .await
                    {
                        error!("Failed to broadcast stock signal for {}: {}", ticker, e);
                    }
                }
                Err(e) => {
                    error!("Error analyzing stock {}: {}", ticker, e);
                }
            }
        }
    }
}

async fn fetch_and_analyze_stock(ticker: &str) -> anyhow::Result<StockSignal> {
    let connector = yahoo::YahooConnector::new();

    if let Ok(connector) = connector {
        // Fetch last 30 days of daily quotes
        let response = connector
            .get_quote_range(ticker, "1d", "30d")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch quotes for {}: {}", ticker, e))?;

        let quotes = response
            .quotes()
            .map_err(|e| anyhow::anyhow!("Failed to parse quotes for {}: {}", ticker, e))?;

        if quotes.is_empty() {
            return Err(anyhow::anyhow!("No quotes found for {}", ticker));
        }

        let last_quote = quotes.last().unwrap();
        let current_price = last_quote.close;

        // Initialize Indicators
        let mut rsi = RelativeStrengthIndex::new(14)?;
        let mut macd = MovingAverageConvergenceDivergence::new(12, 26, 9)?;
        let mut bb = BollingerBands::new(20, 2.0)?;

        let mut rsi_val = 0.0;
        let mut macd_hist = 0.0;
        let mut bb_upper = 0.0;
        let mut bb_lower = 0.0;

        for quote in quotes {
            rsi_val = rsi.next(quote.close);
            let macd_out = macd.next(quote.close);
            macd_hist = macd_out.histogram;
            let bb_out = bb.next(quote.close);
            bb_upper = bb_out.upper;
            bb_lower = bb_out.lower;
        }

        let signal = if rsi_val < 30.0 && macd_hist > 0.0 {
            "STRONG BUY".to_string()
        } else if rsi_val > 70.0 && macd_hist < 0.0 {
            "STRONG SELL".to_string()
        } else {
            "NEUTRAL".to_string()
        };

        return Ok(StockSignal {
            ticker: ticker.to_string(),
            price: current_price,
            rsi: rsi_val,
            macd_hist,
            bb_upper,
            bb_lower,
            signal,
            timestamp: Utc::now().timestamp(),
        });
    }

    return Err(anyhow::anyhow!("No stock found for {}", ticker));
}
