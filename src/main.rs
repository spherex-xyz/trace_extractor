pub mod gather;
pub mod model;
pub mod utils;

use crate::gather::gather;
use ethers_core::types::H256;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use structopt::StructOpt;

#[derive(Deserialize)]
struct ForkedNetwork {
    #[serde(rename = "forkBlockNumber")]
    fork_block_number: u64,
}

#[derive(Deserialize)]
struct HardhatMetadata {
    #[serde(rename = "forkedNetwork")]
    forked_network: Option<ForkedNetwork>,
}

#[derive(Deserialize)]
struct Block {
    transactions: Vec<String>,
}

#[derive(StructOpt)]
struct Cli {
    /// The URL of the Hardhat local network (make sure it is running)
    #[structopt(long, default_value = "http://localhost:8545")]
    url: String,
    /// The starting block number (optional)
    #[structopt(short = "-s", long)]
    start_block: Option<u64>,
    #[structopt(short = "-o", long, default_value = "traces.json")]
    output_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cli {
        url,
        start_block,
        output_path,
    } = Cli::from_args();
    // Create an HTTP client
    let client = Client::new();

    // Determine the starting block number
    let start_block_number = match start_block {
        Some(block) => block - 1,
        None => {
            // Get hardhat_metadata
            let metadata: HardhatMetadata = serde_json::from_value(
                send_request(&client, &url, "hardhat_metadata", json!([])).await?,
            )?;

            metadata
                .forked_network
                .ok_or("Forked network information is not available")?
                .fork_block_number
        }
    };

    // Check for forkBlockNumber
    let latest_block_hex: String =
        serde_json::from_value(send_request(&client, &url, "eth_blockNumber", json!([])).await?)?;

    let latest_block_number =
        u64::from_str_radix(&latest_block_hex.as_str().trim_start_matches("0x"), 16)?;

    // Collect transaction hashes from blocks between forkBlockNumber and latest block number
    let mut transaction_hashes = Vec::new();
    for block_number in (start_block_number + 1)..=latest_block_number {
        let block_param = json!(format!("0x{:x}", block_number));

        let block: Block = serde_json::from_value(
            send_request(
                &client,
                &url,
                "eth_getBlockByNumber",
                json!([block_param, false]),
            )
            .await?,
        )?;

        transaction_hashes.extend(block.transactions);
    }
    let mut traces = Vec::new();

    // Print all collected transaction hashes
    for tx_hash in transaction_hashes {
        let tx_hash_as_hash: H256 = tx_hash.parse()?; // Convert tx_hash from String to H256
        println!("Processing transaction hash: {:?}", tx_hash_as_hash);
        let res = gather(url.to_string(), tx_hash_as_hash).await.unwrap();
        traces.push(res.traces);
    }

    // Serialize the transaction hashes to a JSON string
    let json_hashes = serde_json::to_string_pretty(&traces)?;

    // Write the JSON string to a file
    let mut file = File::create(&output_path)?;
    file.write_all(json_hashes.as_bytes())?;

    Ok(())
}

// Function to send JSON-RPC requests
async fn send_request(
    client: &Client,
    url: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    });

    let response = client.post(url).json(&payload).send().await?;

    let json: serde_json::Value = response.json().await?;

    Ok(json.get("result").ok_or("No result field in response")?).cloned()
}
