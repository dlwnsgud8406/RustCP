use axum::{
    routing::get,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use reqwest::Client;
use std::env;
use anyhow::Result;
use lapin::{options::BasicPublishOptions, BasicProperties, Connection, ConnectionProperties};
use serde_json::json;
use tracing::{info, error};
use tracing_subscriber;
use dotenv::dotenv;
use std::net::SocketAddr;

// Bitget API에서 가져올 데이터 구조
#[derive(Deserialize, Serialize, Debug)]
struct BitgetCandle {
    symbol: String,
    interval: String,
    start_time: i64,
    end_time: i64,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
    market_type: String, // 거래 유형 (spot/futures)
}

// API 수집 핸들러
async fn collect_handler() -> Result<Json<serde_json::Value>, String> {
    // 환경 변수에서 심볼 목록과 거래 유형 읽기
    let symbols_env = env::var("BITGET_SYMBOLS").unwrap_or_else(|_| "BTCUSDT,ETHUSDT".to_string());
    let market_types_env = env::var("BITGET_MARKET_TYPES").unwrap_or_else(|_| "spot,futures".to_string());

    let symbols: Vec<&str> = symbols_env.split(',').collect();
    let market_types: Vec<&str> = market_types_env.split(',').collect();
    let interval = "5m"; // 5분봉

    let client = Client::new();
    let rabbit_host = env::var("RABBITMQ_HOST").unwrap_or_else(|_| "rabbitmq-service".to_string());
    let amqp_url = format!("amqp://admin:admin_password@{}%2f", rabbit_host);

    let conn = Connection::connect(&amqp_url, ConnectionProperties::default())
        .await
        .map_err(|e| e.to_string())?;
    let channel = conn.create_channel().await.map_err(|e| e.to_string())?;

    channel.queue_declare("api_data", Default::default(), Default::default()).await.map_err(|e| e.to_string())?;

    for market_type in market_types {
        for symbol in &symbols {
            let api_url = match market_type {
                "spot" => format!(
                    "https://api.bitget.com/api/spot/v1/market/candles?symbol={}&interval={}",
                    symbol, interval
                ),
                "futures" => format!(
                    "https://api.bitget.com/api/futures/v1/market/candles?symbol={}&interval={}",
                    symbol, interval
                ),
                _ => {
                    error!("Unsupported market type: {}", market_type);
                    continue;
                }
            };

            let response = client.get(&api_url).send().await.map_err(|e| e.to_string())?;
            if !response.status().is_success() {
                error!("Failed to fetch data for symbol {} ({}): {}", symbol, market_type, response.status());
                continue;
            }

            let candles: Vec<BitgetCandle> = response.json().await.map_err(|e| e.to_string())?;

            for mut candle in candles {
                candle.market_type = market_type.to_string();
                let payload = serde_json::to_vec(&candle).map_err(|e| e.to_string())?;
                channel.basic_publish("", "api_data", BasicPublishOptions::default(), &payload, BasicProperties::default())
                    .await
                    .map_err(|e| e.to_string())?;
                info!("Published candle for symbol {} ({}): {:?}", symbol, market_type, candle);
            }
        }
    }
    Ok(Json(json!({"status": "data collected and sent to queue"})))

}

async fn periodic_collect() -> Result<()> {
    loop {
        match collect_handler().await{
            Ok(response) => info!("Periodic collect success: {:?}", response),
            Err(e) => error!("Periodic collect failed: {}", e),
        }
        sleep(Duration::from_secs(300)).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let app = Router::new().route("/collect", get(collect_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    info!("API Collector running on {}", addr);

    tokio::spawn(async {
        if let Err(e) = periodic_collect().await {
            error!("Error in periodic_collect: {}", e);
        }
    });

    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();

    Ok(())
}