use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const API_BASE: &str = "https://ic-api.internetcomputer.org/api/v3/canisters";
const BATCH_SIZE: usize = 100;
const STATE_FILE: &str = "fetcher_state.json";
const OUTPUT_FILE: &str = "canisters.json";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Vec<Canister>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Canister {
    canister_id: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct FetcherState {
    offset: usize,
    canisters: Vec<String>,
}

impl FetcherState {
    fn load() -> Result<Self> {
        if Path::new(STATE_FILE).exists() {
            let contents = fs::read_to_string(STATE_FILE)?;
            Ok(serde_json::from_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(STATE_FILE, json)?;
        Ok(())
    }

    fn clear() -> Result<()> {
        if Path::new(STATE_FILE).exists() {
            fs::remove_file(STATE_FILE)?;
        }
        if Path::new(OUTPUT_FILE).exists() {
            fs::remove_file(OUTPUT_FILE)?;
        }
        Ok(())
    }
}

async fn fetch_batch(client: &reqwest::Client, offset: usize, limit: usize) -> Result<ApiResponse> {
    let url = format!("{}?limit={}&offset={}", API_BASE, limit, offset);
    println!("  Fetching: {}", url);

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("API returned {}: {}", status, body);
    }

    response
        .json::<ApiResponse>()
        .await
        .context("Failed to parse response")
}

fn print_usage() {
    eprintln!("Usage: canister-fetcher [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --count N    Fetch N canisters (default: 1000)");
    eprintln!("  --reset      Clear state and start fresh");
    eprintln!("  --help       Show this help");
    eprintln!();
    eprintln!("State is saved after each batch, so you can resume after interruption.");
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut target_count: usize = 1000;
    let mut reset = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--count" => {
                i += 1;
                target_count = args.get(i)
                    .context("--count requires a number")?
                    .parse()
                    .context("Invalid count")?;
            }
            "--reset" => reset = true,
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    println!("=== Canister ID Fetcher ===\n");

    if reset {
        println!("Resetting state...");
        FetcherState::clear()?;
    }

    // Load existing state (for resume capability)
    let mut state = FetcherState::load().unwrap_or_default();

    if state.offset > 0 {
        println!(
            "Resuming from previous run: {} canisters at offset {}",
            state.canisters.len(),
            state.offset
        );
    }

    println!("Target: {} canisters\n", target_count);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut batch_num = 0;

    while state.canisters.len() < target_count {
        batch_num += 1;
        println!(
            "Batch {}: fetching {} canisters at offset {}...",
            batch_num, BATCH_SIZE, state.offset
        );

        match fetch_batch(&client, state.offset, BATCH_SIZE).await {
            Ok(response) => {
                let count = response.data.len();
                println!("  Got {} canisters", count);

                if count == 0 {
                    println!("No more canisters available!");
                    break;
                }

                // Extract canister IDs
                for canister in &response.data {
                    state.canisters.push(canister.canister_id.clone());
                }

                state.offset += count;

                // Save state after each batch (for resume on failure)
                state.save().context("Failed to save state")?;

                println!(
                    "  Progress: {}/{} (next offset: {})",
                    state.canisters.len(),
                    target_count,
                    state.offset
                );

                // Small delay to be nice to the API
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            Err(e) => {
                println!("  ERROR: {}", e);
                println!("  State saved - you can resume by running again");
                state.save()?;
                return Err(e);
            }
        }
    }

    // Save final output
    let output_json = serde_json::to_string_pretty(&state.canisters)?;
    fs::write(OUTPUT_FILE, &output_json)?;

    println!("\n=== Complete ===");
    println!("Total canisters fetched: {}", state.canisters.len());
    println!("Output saved to: {}", OUTPUT_FILE);
    println!("State saved to: {} (delete to start fresh)", STATE_FILE);

    // Show first and last few IDs
    println!("\nFirst 3 canister IDs:");
    for id in state.canisters.iter().take(3) {
        println!("  {}", id);
    }
    if state.canisters.len() > 6 {
        println!("\nLast 3 canister IDs:");
        for id in state.canisters.iter().rev().take(3).collect::<Vec<_>>().into_iter().rev() {
            println!("  {}", id);
        }
    }

    Ok(())
}
