use simple_logger::SimpleLogger;
use log::info;
use std::sync::mpsc;
use std::thread;
use clap::Parser;
use serde_json::Value;

use agent::model::{FirestarterParams, RaplRecord, ServerInfo};
use agent::bmc::monitor_bmc::monitor_bmc;

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


#[allow(clippy::upper_case_acronyms)]
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct CLI {
    // Passing default values here for the tests - to to deleted
    #[arg(long, short = 'H', name = "host")]
    bmc_hostname: String,

    #[arg(long, short = 'U', name = "user")]
    bmc_username: String,

    #[arg(long, short = 'P', name = "password")]
    bmc_password: String,

    #[arg(
        long,
        default_value_t = 10,
        name = "warmup seconds",
        help = "Number of seconds to warm up before applying cap"
    )]
    warmup: u64,

    #[arg(
        long,
        short,
        default_value_t = 15,
        name = "test time seconds",
        help = "Number of seconds to wait after applying a cap before testing if cap has been applied. "
    )]
    test_time: u64,

    #[arg(
        long = "cap_low",
        short = 'w',
        default_value_t = 400,
        name = "low watts",
        help = "Number of Watts for setting a low cap"
    )]
    cap_low_watts: u64,

    #[arg(
        long = "cap_high",
        short = 'W',
        default_value_t = 580,
        name = "high watts",
        help = "Number of Watts for setting a high cap, used before setting a low cap"
    )]
    cap_high_watts: u64,

    #[arg(
        long,
        short,
        default_value = "./stats",
        name = "stats directory",
        help = "Directory to store runtime stats in"
    )]
    stats_dir: String,

    #[arg(
        long,
        default_value = "/home_nfs/wainj/local/bin/firestarter",
        name = "firestarter path",
        help = "Path to firestarter executable (relative or absolute)"
    )]
    firestarter: String,

    #[arg(
        long,
        default_value = "/usr/bin/ipmitool",
        name = "ipmi path",
        help = "Path to ipmi executable (relative or absolute)"
    )]
    ipmi: String,
}
