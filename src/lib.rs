pub mod handlers;
pub mod model;
pub mod route;
pub mod server;
pub mod rapl;
pub mod firestarter;
pub mod bmc;
pub mod test;

use std::os::unix::fs::MetadataExt;
use clap::Parser;
use chrono::Local;
use lazy_static::lazy_static;


fn am_root() -> bool {
    let uid = std::fs::metadata("/proc/self").map(|m| m.uid())
        .expect("Failed to read /proc/self");
    println!("UID: {uid}");
    uid == 0
}

const SETUP_PAUSE_MILLIS: u64 = 300;
const AGENT_INFO_ENDPOINT: &str = "http://oahu10000:8000/api/system_info";
const AGENT_RUN_TEST_ENDPOINT: &str = "http://oahu10000:8000/api/run_test";

lazy_static! {
    /*
        Global configuration variable.

        Lazy-static creates singleton (one-off) types that wraps a value
        providing single initialization and thread-safety.

        For a given: static ref NAME: TYPE = EXPR;
        The lazy_static macro creates a unique type that implements
        Deref<TYPE> and stores it in a static with name NAME.

        It is the wrapped value that implements any traits (eg Debug, Clone),
        NOT the wrapper. Because of this, must deref (*NAME) when debug/trace
        printing.
    */

    pub static ref CONFIGURATION: Configuration = Configuration::new();
}

#[derive(Debug)]
pub struct Configuration {
    pub bmc_hostname: String,
    pub bmc_username: String,
    pub bmc_password: String,
    pub warmup_secs: u64,
    pub test_time_secs: u64,
    pub cap_low_watts: u64,
    pub cap_high_watts: u64,
    pub stats_dir: String,
    pub test_timestamp: String,
    pub firestarter: String,
    pub ipmi: String,
    pub setup_pause_millis: u64,
    pub agent_info_endpoint: String,
    pub agent_run_test_endpoint: String,
}

impl Configuration {
    fn new() -> Self {
        let args = CLI::parse();
        let timestamp_format = "%y%m%d_%H%M";
        let test_timestamp = Local::now().format(timestamp_format).to_string();

        Configuration {
            bmc_hostname: args.bmc_hostname,
            bmc_username: args.bmc_username,
            bmc_password: args.bmc_password,
            warmup_secs: args.warmup,
            test_time_secs: args.test_time,
            cap_low_watts: args.cap_low_watts,
            cap_high_watts: args.cap_high_watts,
            stats_dir: args.stats_dir,
            test_timestamp,
            firestarter: args.firestarter,
            ipmi: args.ipmi,
            setup_pause_millis: SETUP_PAUSE_MILLIS,
            agent_info_endpoint: String::from(AGENT_INFO_ENDPOINT),
            agent_run_test_endpoint: String::from(AGENT_RUN_TEST_ENDPOINT),
        }
    }
}

/*
    >>> ATTENTION <<<

    When updating the CLI structure below, you'll probably want to
    update the Configuration structure (and its implementation) too.
*/

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
