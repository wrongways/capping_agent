use simple_logger::SimpleLogger;
use log::{trace, info};
use std::sync::mpsc::{self, Receiver};
use std::path::{Path, PathBuf};
use std::fs::{self, OpenOptions};
use std::thread; // OK to mix threads with Tokio
use tokio::task;
use tokio::time::{Duration, sleep};
use reqwest::Client;
use chrono::Utc;
use serde::Serialize;

use agent::Timestamps;
use agent::model::{FirestarterParams, RaplRecord, ServerInfo};
use agent::bmc::monitor_bmc::monitor_bmc;
use agent::bmc::{bmc::BMC, BMCStats};
use agent::test::{load_iterator::LoadTestSuite, thread_iterator::ThreadTestSuite, Test, TestRun, CappingOrder, Operation, TestSuiteInfo, CapStep};
use agent::CONFIGURATION;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().env().init()?;

    let client = make_http_client();
    let bmc = make_bmc();

    let server_info = get_server_info(&client).await;
    info!("Host info:\n{server_info:?}");

    // buffers to hold the collected statistics
    let mut runs: Vec<TestRun> = Vec::new();
    let mut all_bmc_stats: Vec<Vec<BMCStats>> = Vec::new();
    let mut all_rapl_stats: Vec<Vec<RaplRecord>> = Vec::new();


    let load_tests = LoadTestSuite::new();
    let thread_tests = ThreadTestSuite::new(server_info.system_info.online_cpus);

    // Calculate extra time required for stepped tests

    // How many steps are required?
    let cap_power_delta = CONFIGURATION.cap_high_watts - CONFIGURATION.cap_low_watts;
    let cap_step_count = (cap_power_delta as f64 / CONFIGURATION.cap_step_size_watts as f64).ceil() as u64;
    let step_time = (cap_step_count + 1) * CONFIGURATION.cap_step_interval_secs;

    let mut total_runtime_secs = CONFIGURATION.warmup_secs + CONFIGURATION.test_time_secs;


    for test in load_tests {
        info!("{test:?}");

        // Only run cap down tests (in the interests of execution time)
        if test.cap_from > test.cap_to {
            if test.step == CapStep::Step {
                total_runtime_secs += step_time;
            }

            let (rapl_stats, bmc_stats, timestamps) = run_test(&test, total_runtime_secs, &client, &bmc).await?;
            let (start_timestamp, cap_timestamp, end_timestamp) = timestamps;

            info!("RAPL stats\n{rapl_stats:?}");
            info!("BMC Stats\n{bmc_stats:?}");
            info!("Start, cap, end timestamps: {start_timestamp}, {cap_timestamp}, {end_timestamp}");

            let test_run = TestRun::new(timestamps, test);
            runs.push(test_run);
            all_bmc_stats.push(bmc_stats);
            all_rapl_stats.push(rapl_stats);
        }
    }

    for test in thread_tests {
        info!("{test:?}");

        // Only run cap down tests (in the interests of execution time)
        if test.cap_from > test.cap_to {
            if test.step == CapStep::Step {
                total_runtime_secs += step_time;
            }

            let (rapl_stats, bmc_stats, timestamps) = run_test(&test, total_runtime_secs, &client, &bmc).await?;
            let (start_timestamp, cap_timestamp, end_timestamp) = timestamps;

            info!("RAPL stats\n{rapl_stats:?}");
            info!("BMC Stats\n{bmc_stats:?}");
            info!("Start, cap, end timestamps: {start_timestamp}, {cap_timestamp}, {end_timestamp}");

            let test_run = TestRun::new(timestamps, test);
            runs.push(test_run);
            all_bmc_stats.push(bmc_stats);
            all_rapl_stats.push(rapl_stats);
        }
    }

    // All done, so OK to pass ownership here
    save_logs(runs, all_rapl_stats, all_bmc_stats);
    log_server_info(server_info);

    Ok(())
}

async fn run_test(config: &Test, runtime_secs: u64, client: &Client, bmc: &BMC) ->
    Result<(Vec<RaplRecord>, Vec<BMCStats>, Timestamps), Box<dyn std::error::Error>> {

    trace!("Running test: {config:?}");

    let start_timestamp = Utc::now();
    let (bmc_tx, bmc_rx) = mpsc::channel();
    let bmc_thread = start_bmc_monitor(bmc_rx);

    let fs_params = FirestarterParams {
        runtime_secs,
        load_pct: config.load_pct,
        load_period_us: config.load_period,
        n_threads: config.n_threads
    };

    trace!("Setting initial conditions");
    set_initial_conditions(config, bmc).await;
    trace!("launching agent");
    let agent_thread = launch_agent(client.clone(), fs_params);

    //  Wait for warmup seconds
    trace!("Sleeping for warmup period");
    sleep(Duration::from_secs(CONFIGURATION.warmup_secs)).await;
    trace!("Doing cap_operation");
    let cap_timestamp = Utc::now();
    do_cap_operation(config, bmc);

    trace!("Joining agent thread (firestarter exit)");
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
    trace!("Sending request to agent: {}", &CONFIGURATION.agent_run_test_endpoint);
    task::spawn(async move {
        trace!("Agent thread posting to {}", &CONFIGURATION.agent_run_test_endpoint);
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
    trace!("starting setup_initial_conditions()");
    match config.capping_order {
        CappingOrder::LevelBeforeActivate => {
            // Set the level to the "cap_to" value, and the
            // capping activation to the opposite of the test

            bmc.set_cap_power_level(config.cap_to);
            // sleep(Duration::from_secs(CONFIGURATION.setup_pause_millis));

            match config.operation {
                Operation::Activate => bmc.deactivate_power_cap(),
            };
        }
        CappingOrder::LevelAfterActivate => {
            // set the capping level to the "cap_from" value
            // and the capping activation to the value for the test
            bmc.set_cap_power_level(config.cap_from);
            // sleep(Duration::from_secs(CONFIGURATION.setup_pause_millis));

            match config.operation {
                Operation::Activate => bmc.activate_power_cap(),
            }
        }

    };
    trace!("initial_conditions set - pause before return");
    // sleep(Duration::from_secs(CONFIGURATION.setup_pause_millis)).await;
    trace!("exiting setup_initial_conditions()");
}

fn do_cap_operation(config: &Test, bmc: &BMC) {
    trace!("do_cap_operation()");
    match config.capping_order {
        CappingOrder::LevelBeforeActivate => {
            // The capping level is set by set_initial_conditions
            // just need to perform the operation
            match config.operation {
                Operation::Activate => bmc.activate_power_cap(),
            }
        }
        CappingOrder::LevelAfterActivate => {
            if config.step == CapStep::OneShot {
                bmc.set_cap_power_level(config.cap_to);
            } else {
                // Step up/down the cap
                let mut current_cap = config.cap_from;

                // ensure step size won't overshoot the cap_to value
                while (current_cap as i64 - config.cap_to as i64).unsigned_abs() > CONFIGURATION.cap_step_size_watts {
                    if config.cap_from > config.cap_to {
                        current_cap -= CONFIGURATION.cap_step_size_watts;
                    } else {
                        current_cap += CONFIGURATION.cap_step_size_watts;
                    }
                    bmc.set_cap_power_level(current_cap);
                    thread::sleep(Duration::from_secs(CONFIGURATION.cap_step_interval_secs));
                }
                // set the final value
                bmc.set_cap_power_level(config.cap_to);
            }
        }
    }
}


async fn get_server_info(client: &Client ) -> ServerInfo {
    trace!("get_server_info endpoint: {}", &CONFIGURATION.agent_info_endpoint);
    client.get(&CONFIGURATION.agent_info_endpoint)
    .send()
    .await
    .expect("Failed to get server info")
    .json()
    .await
    .expect("Failed to get JSON from ServerInfo")
}


fn save_logs(tests: Vec<TestRun>, rapl_stats: Vec<Vec<RaplRecord>>, bmc_stats: Vec<Vec<BMCStats>>) {
    // create the stats directory
    let stats_path = Path::new(&CONFIGURATION.stats_dir);
    fs::create_dir_all(stats_path).expect("Failed to create stats directory");

    let mut path = PathBuf::from(stats_path);
    path.push(format!("test_config_{}.json", CONFIGURATION.log_timestamp()));
    write_json_file(&path, &tests);

    path = PathBuf::from(stats_path);
    path.push(format!("bmc_stats_{}.json", CONFIGURATION.log_timestamp()));
    write_json_file(&path, &bmc_stats);

    path = PathBuf::from(stats_path);
    path.push(format!("rapl_stats_{}.json", CONFIGURATION.log_timestamp()));
    write_json_file(&path, &rapl_stats);
}

fn log_server_info(server_info: ServerInfo)  {
    let stats_path = PathBuf::from(
        &format!("{}/server_info_{}.json",
        &CONFIGURATION.stats_dir,
        CONFIGURATION.log_timestamp()));

    let test_suite_info = TestSuiteInfo {
        start_timestamp: CONFIGURATION.test_start_timestamp,
        end_timestamp: Utc::now(),
        server_info,
    };
    write_json_file(&stats_path, &test_suite_info);

}

fn write_json_file<T>(path: &PathBuf, json: &T) where T: ?Sized + Serialize {
    let handle = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .expect("Failed to open file");
    serde_json::to_writer_pretty(&handle, json).expect("Failed to write json");
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
