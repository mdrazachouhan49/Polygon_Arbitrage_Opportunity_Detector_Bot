Polygon Arbitrage Opportunity Detector Bot
This is a Rust application designed to detect potential arbitrage opportunities on the Polygon blockchain network. The bot monitors the prices of a specific token pair across two different Decentralized Exchanges (DEXes) and identifies profitable discrepancies, logging them to a local database.

The project is built on the foundation of blockchain interaction in Rust, demonstrating how to query on-chain data and perform calculations to identify market inefficiencies.

Key Features:
Multi-DEX Price Fetching: Connects to a Polygon RPC node to get real-time price data from two DEXes (Uniswap and QuickSwap).
Arbitrage Detection: Compares prices and identifies opportunities where the profit exceeds a user-defined threshold.
Simulated Profit Calculation: Calculates estimated profit for a hypothetical trade, accounting for a simplified gas cost.
Configuration Management: Uses config.toml and .env files for easy and secure configuration of RPC URLs, contract addresses, and trade parameters.
Data Persistence: Logs all detected opportunities and their simulated profits to an SQLite database.

Technology Stack:
Programming Language: Rust
Blockchain: Polygon Network
DEX Interaction: Uniswap V3 and QuickSwap V2
Tokens: WETH and USDC
Libraries: ethers-rs, tokio, anyhow, dotenv, rusqlite, serde, toml

Getting Started:
Follow these steps to set up and run the bot on your local machine.

1.Prerequisites
Rust and Cargo: Ensure you have a working Rust development environment. If not, install rustup from the official Rust website.

2.Installation
Clone the repository:

git clone (https://github.com/mdrazachouhan49/Polygon_Arbitrage_Opportunity_Detector_Bot.git)
cd polygon-arbitrage-bot

3.Set up configuration files:

Create a .env file for your RPC URL. You can use a public RPC for testing, but a private one is recommended for stability.

POLYGON_RPC_URL="https://polygon-rpc.com"

The config.toml file contains the DEX and token addresses. This file is already configured with the correct addresses for WETH and USDC on Uniswap V3 and QuickSwap V2.

4.Running the Bot
Build the project: This command will also run the build.rs script to generate ABI bindings for the smart contracts.

cargo build

5.Run the application:

cargo run

The bot will start running in your terminal, periodically checking prices and logging any detected arbitrage opportunities.



Project Structure:
src/main.rs: Contains the core application logic, including the price fetching loop, arbitrage detection, and database logging.

build.rs: A build script that generates Rust-friendly bindings from the DEX smart contract ABIs.

Cargo.toml: The project manifest file that lists all dependencies and build configurations.

config.toml: Configuration file for DEX addresses, token addresses, and arbitrage parameters.

.env: Secure file for storing sensitive information like the RPC URL.

arbitrage_opportunities.db: The SQLite database file where opportunities are logged. (This file is ignored by Git).

.gitignore: Specifies files and directories that Git should ignore.