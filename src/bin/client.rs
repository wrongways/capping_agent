use simple_logger::SimpleLogger;
use log::info;
use std::sync::mpsc::{self, Receiver};
use tokio::task;
use tokio::time::{sleep, Duration};
use reqwest::Client;
use chrono::{Utc, DateTime};

use agent::model::{FirestarterParams, RaplRecord, ServerInfo};
use agent::bmc::monitor_bmc::monitor_bmc;
use agent::bmc::{bmc::BMC, BMCStats};
use agent::test::{load_iterator::LoadTestSuite, thread_iterator::ThreadTestSuite, Test, CappingOrder, Operation};
use agent::CONFIGURATION;


type Timestamps = (DateTime<Utc>, DateTime<Utc>, DateTime<Utc>);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().env().init()?;

    let client = make_http_client();
    let bmc = make_bmc();

    let server_info = get_server_info(&client).await;
    info!("Host info:\n{server_info:?}");


    let load_tests = LoadTestSuite::new();
    let thread_tests = ThreadTestSuite::new(server_info.system_info.online_cpus);

    let total_runtime_secs = CONFIGURATION.warmup_secs + CONFIGURATION.test_time_secs;



    for test in load_tests {
        println!("{test:?}");
    }

    for test in thread_tests {
        info!("{test:?}");
        let (rapl_stats, bmc_stats, timestamps) = run_test(&test, total_runtime_secs, &client, &bmc).await?;
        let (start, cap, end) = timestamps;

        info!("RAPL stats\n{rapl_stats:?}");
        info!("BMC Stats\n{bmc_stats:?}");
        info!("Start, cap, end timestamps: {start}, {cap}, {end}");

        log_results(&server_info, &test, &rapl_stats, &bmc_stats, &timestamps);
    }

    Ok(())
}

async fn run_test(config: &Test, runtime_secs: u64, client: &Client, bmc: &BMC) ->
    Result<(Vec<RaplRecord>, Vec<BMCStats>, Timestamps), Box<dyn std::error::Error>> {

    let start_timestamp = Utc::now();
    let (bmc_tx, bmc_rx) = mpsc::channel();
    let bmc_thread = start_bmc_monitor(bmc_rx);

    let fs_params = FirestarterParams {
        runtime_secs,
        load_pct: config.load_pct,
        load_period_us: config.load_period,
        n_threads: config.n_threads
    };


    set_initial_conditions(config, bmc).await;
    let agent_thread = launch_agent(client.clone(), fs_params);

    //  Wait for warmup seconds
    sleep(Duration::from_secs(CONFIGURATION.warmup_secs)).await;
    let cap_timestamp = Utc::now();
    do_cap_operation(config, bmc);


    let rapl_stats: Vec<RaplRecord> = agent_thread.await.expect("");

    bmc_tx.send(()).expect("Failed to signal BMC thread");
    let bmc_stats: Vec<BMCStats> = bmc_thread.await.expect("Failed to join BMC thread");
    let end_timestamp = Utc::now();

    Ok((rapl_stats, bmc_stats, (start_timestamp, cap_timestamp, end_timestamp)))
}

fn start_bmc_monitor(rx_channel: Receiver<()>) -> task::JoinHandle<Vec<BMCStats>> {
    task::spawn(async {monitor_bmc(rx_channel)})
}

fn launch_agent(client: Client, fs_params: FirestarterParams ) ->  task::JoinHandle<Vec<RaplRecord>> {
    task::spawn(async move {
        client
            .post(&CONFIGURATION.agent_run_test_endpoint)
            .json(&fs_params)
            .send()
            .await
            .expect("launch_agent failed to post request")
            .json()
            .await
            .expect("launch_agent failed to extract json")
    })
}


async fn set_initial_conditions(config: &Test, bmc: &BMC) {
    match config.capping_order {
        CappingOrder::LevelBeforeActivate => {
            // Set the level to the "cap_to" value, and the
            // capping activation to the opposite of the test

            bmc.set_cap_power_level(config.cap_to);
            sleep(Duration::from_secs(CONFIGURATION.setup_pause_millis)).await;

            match config.operation {
                Operation::Activate => bmc.deactivate_power_cap(),
                Operation::Deactivate => bmc.activate_power_cap(),
            };
        }
        CappingOrder::LevelAfterActivate => {
            // set the capping level to the "cap_from" value
            // and the capping activation to the value for the test
            bmc.set_cap_power_level(config.cap_from);
            sleep(Duration::from_secs(CONFIGURATION.setup_pause_millis)).await;

            match config.operation {
                Operation::Activate => bmc.activate_power_cap(),
                Operation::Deactivate => bmc.deactivate_power_cap(),
            }
        }
        CappingOrder::LevelToLevel | CappingOrder::LevelToLevelActivate => {
            // set cap level and activate capping
            bmc.set_cap_power_level(config.cap_from);
            sleep(Duration::from_secs(CONFIGURATION.setup_pause_millis)).await;
            bmc.activate_power_cap();
        }
    };
    sleep(Duration::from_secs(CONFIGURATION.setup_pause_millis)).await;
}

fn do_cap_operation(config: &Test, bmc: &BMC) {
    match config.capping_order {
        CappingOrder::LevelBeforeActivate => {
            // The capping level is set by set_initial_conditions
            // just need to perform the operation
            match config.operation {
                Operation::Activate => bmc.activate_power_cap(),
                Operation::Deactivate => bmc.deactivate_power_cap(),
            }
        }
        CappingOrder::LevelAfterActivate | CappingOrder::LevelToLevel => {
            bmc.set_cap_power_level(config.cap_to);
        }
        CappingOrder::LevelToLevelActivate => {
            bmc.set_cap_power_level(config.cap_to);
            bmc.activate_power_cap();
        }
    }
}


async fn get_server_info(client: &Client ) -> ServerInfo {
    client.get(&CONFIGURATION.agent_info_endpoint)
    .send()
    .await
    .expect("Failed to get server info")
    .json()
    .await
    .expect("Failed to get JSON from ServerInfo")
}


fn log_results(server_info: &ServerInfo, config: &Test, rapl_stats: &Vec<RaplRecord>, bmc_stats: &Vec<BMCStats>, timestamps: &Timestamps) {

}


fn make_bmc() -> BMC {
    BMC::new(
        &CONFIGURATION.bmc_hostname,
        &CONFIGURATION.bmc_username,
        &CONFIGURATION.bmc_password,
        &CONFIGURATION.ipmi
    )
}

fn make_http_client() -> Client {
    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
    );

    reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("Failed to create a client")
}
