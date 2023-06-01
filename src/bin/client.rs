use simple_logger::SimpleLogger;
use log::info;
use std::sync::mpsc;
use std::thread;
use clap::Parser;
use serde_json::Value;

use agent::model::{FirestarterParams, RaplRecord};
use agent::bmc::monitor_bmc::monitor_bmc;
use agent::CLI;
use agent::test::load_iterator::TestSuite;

const AGENT_INFO_ENDPOINT: &str = "http://oahu10000:8000/api/system_info";
const AGENT_RUN_TEST_ENDPOINT: &str = "http://oahu10000:8000/api/run_test";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().env().init()?;

    let args = CLI::parse();

    let host_info: Value = reqwest::get(AGENT_INFO_ENDPOINT)
        .await?
        .json()
        .await?;

    info!("Host info:\n{host_info:?}");

    let _load_iterator = TestSuite::new();

    // Start BMC monitor
    let (bmc_tx, bmc_rx) = mpsc::channel();
    let bmc_thread = thread::spawn(move ||
        monitor_bmc(
            &args.bmc_hostname,
            &args.bmc_username,
            &args.bmc_password,
            &args.ipmi,
            bmc_rx
        )
    );

    let fs_params = FirestarterParams {
        runtime_secs: 20,
        load_pct: 100,
        load_period_us: 0,
        n_threads: 0,
    };

    let rapl_stats: Vec<RaplRecord> = reqwest::Client::new()
        .post(AGENT_RUN_TEST_ENDPOINT)
        .json(&fs_params)
        .send()
        .await?
        .json()
        .await?;

    bmc_tx.send(()).expect("Failed to signal BMC thread");
    let bmc_stats = bmc_thread.join().expect("Failed to join BMC thread");

    info!("RAPL stats\n{rapl_stats:?}");
    info!("\n\nBMC Stats\n{bmc_stats:?}");

    Ok(())
}


