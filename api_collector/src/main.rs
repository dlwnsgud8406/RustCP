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
    
}