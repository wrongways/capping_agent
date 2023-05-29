use simple_logger::SimpleLogger;
use log::info;

use agent::model::FirestarterParams;

const AGENT_INFO_ENDPOINT: &str = "http://oahu10000:8000/api/system_info";
const AGENT_RUN_TEST_ENDPOINT: &str = "http://oahu10000:8000/api/run_test";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().env().init()?;
    let host_info = reqwest::get(AGENT_INFO_ENDPOINT)
        .await?
        .text()
        .await?;

    info!("Host info:\n{host_info:?}");

    let fs_params = FirestarterParams {
        runtime_secs: 20,
        load_pct: 100,
        load_period_us: 0,
        n_threads: 0,
    };

    let rapl_stats = reqwest::Client::new()
        .post(AGENT_RUN_TEST_ENDPOINT)
        .json(&fs_params)
        .send()
        .await?;

    info!("RAPL stats\n{rapl_stats:?}");

    Ok(())
}
