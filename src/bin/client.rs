use simple_logger::SimpleLogger;
use log::info;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use agent::model::{FirestarterParams, RaplRecord, ServerInfo};
use agent::bmc::monitor_bmc::monitor_bmc;
use agent::bmc::BMCStats;
use agent::test::{load_iterator::LoadTestSuite, thread_iterator::ThreadTestSuite, Test};
use agent::CONFIGURATION;


const AGENT_INFO_ENDPOINT: &str = "http://oahu10000:8000/api/system_info";
const AGENT_RUN_TEST_ENDPOINT: &str = "http://oahu10000:8000/api/run_test";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().env().init()?;

    let host_info: ServerInfo = reqwest::get(AGENT_INFO_ENDPOINT)
        .await?
        .json()
        .await?;

    info!("Host info:\n{host_info:?}");


    let load_tests = LoadTestSuite::new();
    let thread_tests = ThreadTestSuite::new(host_info.system_info.online_cpus);

    let total_runtime_secs = CONFIGURATION.warmup_secs + CONFIGURATION.test_time_secs;

    for test in load_tests {
        println!("{test:?}");
    }

    for test in thread_tests {
        println!("{test:?}");
        let (rapl_stats, bmc_stats) = run_test(test, total_runtime_secs).await?;
        info!("RAPL stats\n{rapl_stats:?}");
        info!("\n\nBMC Stats\n{bmc_stats:?}");
    }

    Ok(())
}

async fn run_test(config: Test, runtime_secs: u64) ->
    Result<(Vec<RaplRecord>, Vec<BMCStats>), Box<dyn std::error::Error>> {

    let (bmc_tx, bmc_rx) = mpsc::channel();
    let bmc_thread = start_bmc_monitor(bmc_rx);

    let fs_params = FirestarterParams {
        runtime_secs,
        load_pct: config.load_pct,
        load_period_us: config.load_period,
        n_threads: config.n_threads
    };

    let rapl_stats: Vec<RaplRecord> = reqwest::Client::new()
        .post(AGENT_RUN_TEST_ENDPOINT)
        .json(&fs_params)
        .send()
        .await?
        .json()
        .await?;

    bmc_tx.send(()).expect("Failed to signal BMC thread");
    let bmc_stats: Vec<BMCStats> = bmc_thread.join().expect("Failed to join BMC thread");

    Ok((rapl_stats, bmc_stats))
}

fn start_bmc_monitor(rx_channel: Receiver<()>) -> thread::JoinHandle<Vec<BMCStats>> {
    thread::spawn(move ||
        monitor_bmc(rx_channel)
    )
}

