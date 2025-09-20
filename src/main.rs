
include!(concat!(env!("OUT_DIR"), "/uniswap_v2_router.rs"));

use anyhow::Result;
use dotenv::dotenv;
use ethers_core::{
    types::{Address, U256},
    utils::parse_units,
};
use ethers_providers::{Http, Provider};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use chrono::Local;

// Deserialize the configuration file
#[derive(Debug, Deserialize)]
struct Config {
    rpc_url_key: String,
    dexes: DexesConfig,
    tokens: TokensConfig,
    arbitrage: ArbitrageConfig,
}

#[derive(Debug, Deserialize)]
struct DexesConfig {
    uniswap_v2: Address,
    quickswap_v2: Address,
}

#[derive(Debug, Deserialize)]
struct TokensConfig {
    weth: Address,
    usdc: Address,
}

#[derive(Debug, Deserialize)]
struct ArbitrageConfig {
    trade_amount_weth: f64,
    profit_threshold_usdc: f64,
    polling_interval_seconds: u64,
}

// Function to set up the SQLite database
fn setup_database() -> SqliteResult<Connection> {
    let db_path = "arbitrage_opportunities.db";
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS opportunities (
            id INTEGER PRIMARY KEY,
            timestamp TEXT NOT NULL,
            profit_usdc REAL NOT NULL,
            buy_dex TEXT NOT NULL,
            sell_dex TEXT NOT NULL,
            buy_price_usdc_per_weth REAL NOT NULL,
            sell_price_usdc_per_weth REAL NOT NULL
        )",
        [],
    )?;

    println!("✅ Database setup complete: {}", db_path);
    Ok(conn)
}

async fn fetch_price<M: ethers_providers::Middleware>(
    contract: &UniswapRouter<M>,
    amount_in: U256,
    path: Vec<Address>,
    dex_name: &str,
) -> Result<U256> {
    // Add this debug print
    println!(
        "📊 Fetching price from {} with amount_in: {} wei, path: {:?}",
        dex_name, amount_in, path
    );
     
    match contract.get_amounts_out(amount_in, path).call().await {
        Ok(amounts) => Ok(amounts[1]),
        Err(e) => {
            eprintln!("Error fetching price from {}: {:?}", dex_name, e);
            Ok(U256::zero())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load Configuration
    dotenv().ok(); // Load .env file
    let config_str = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_str)?;

    let rpc_url = std::env::var(&config.rpc_url_key)?;
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    // 2. Setup Database
    let db_conn = setup_database()?;

    // Main loop for periodic checks
    loop {
        println!("🔍 Checking for arbitrage opportunities...");

        // Get contract and token addresses from config
        let dex_1_addr = config.dexes.uniswap_v2;
        let dex_2_addr = config.dexes.quickswap_v2;
        let weth_addr = config.tokens.weth;
        let usdc_addr = config.tokens.usdc;

        // Corrected line: explicit type conversion using .into()
    
        let buy_amount_weth_u256: U256 = parse_units(config.arbitrage.trade_amount_weth, 18).unwrap().into(); // WETH has 18 decimals

        // 3. Fetch Prices from DEXes
        // Create contract instances with the generated bindings
        let dex_1_contract = UniswapRouter::new(dex_1_addr, client.clone());
        let dex_2_contract = UniswapRouter::new(dex_2_addr, client.clone());

        // Call getAmountsOut on both DEXes
        let path_weth_usdc: Vec<Address> = vec![weth_addr, usdc_addr];
        let path_usdc_weth: Vec<Address> = vec![usdc_addr, weth_addr];

        // Get price on DEX 1 (WETH -> USDC)
        let price_1_usdc = fetch_price(&dex_1_contract, buy_amount_weth_u256, path_weth_usdc.clone(), "Uniswap V2").await?;

        // Get price on DEX 2 (USDC -> WETH)
        let price_2_weth = fetch_price(&dex_2_contract, price_1_usdc, path_usdc_weth.clone(), "QuickSwap V2").await?;

        
        // 4. Calculate Simulated Profit
        // USDC has 6 decimals, WETH has 18.
        let weth_to_usdc_price_1 =
            price_1_usdc.as_u128() as f64 / 1_000_000.0 / config.arbitrage.trade_amount_weth;
        // This is a direct conversion, not a second swap, so the price is just the ratio.
        let usdc_to_weth_price_2 =
            price_2_weth.as_u128() as f64 / 1_000_000.0 / (price_1_usdc.as_u128() as f64 / 1_000_000.0);

        // Simulated gas cost in USDC
        let simulated_gas_cost_usdc = 0.0; // a fixed, simplified cost

        // Corrected profit calculation
        let profit_weth_abs_diff = if price_2_weth > buy_amount_weth_u256 {
            price_2_weth - buy_amount_weth_u256
        } else {
            U256::zero()
        };

        let simulated_profit = (profit_weth_abs_diff.as_u128() as f64 / 1_000_000_000_000_000_000.0) * weth_to_usdc_price_1 - simulated_gas_cost_usdc;

        // 5. Detect and Log Opportunity
        if simulated_profit > config.arbitrage.profit_threshold_usdc {
            let buy_dex = "Uniswap V3";
            let sell_dex = "QuickSwap V2";
            let buy_price = 1.0 / weth_to_usdc_price_1;
            let sell_price = usdc_to_weth_price_2;

            println!(
                "💰 Potential Arbitrage Opportunity Found! Profit: {:.4} USDC",
                simulated_profit
            );
            println!("  - Buy WETH on {}: {:.6} USDC/WETH", buy_dex, buy_price);
            println!("  - Sell WETH on {}: {:.6} USDC/WETH", sell_dex, sell_price);

            // Insert into database
            db_conn.execute(
                "INSERT INTO opportunities (timestamp, profit_usdc, buy_dex, sell_dex, buy_price_usdc_per_weth, sell_price_usdc_per_weth) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    Local::now().to_string(),
                    simulated_profit,
                    buy_dex,
                    sell_dex,
                    buy_price,
                    sell_price
                ],
            )?;

            println!("  - Logged to database.");
        } else {
            println!(
                "📉 No significant arbitrage opportunity found. (Profit: {:.4} USDC)",
                simulated_profit
            );
        }

        // 6. Wait for next polling interval
        sleep(Duration::from_secs(
            config.arbitrage.polling_interval_seconds,
        ))
        .await;
    }
}