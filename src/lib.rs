pub mod handlers;
pub mod model;
pub mod response;
pub mod route;
pub mod server;
pub mod rapl;
pub mod firestarter;
pub mod bmc;
pub mod test;

use std::os::unix::fs::MetadataExt;
use clap::Parser;


fn am_root() -> bool {
    let uid = std::fs::metadata("/proc/self").map(|m| m.uid())
        .expect("Failed to read /proc/self");
    println!("UID: {uid}");
    uid == 0
}


#[allow(clippy::upper_case_acronyms)]
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
pub struct CLI {
    // Passing default values here for the tests - to to deleted
    #[arg(long, short = 'H', name = "host")]
    pub bmc_hostname: String,

    #[arg(long, short = 'U', name = "user")]
    pub bmc_username: String,

    #[arg(long, short = 'P', name = "password")]
    pub bmc_password: String,

    #[arg(
        long,
        default_value_t = 10,
        name = "warmup seconds",
        help = "Number of seconds to warm up before applying cap"
    )]
    pub warmup: u64,

    #[arg(
        long,
        short,
        default_value_t = 15,
        name = "test time seconds",
        help = "Number of seconds to wait after applying a cap before testing if cap has been applied. "
    )]
    pub test_time: u64,

    #[arg(
        long = "cap_low",
        short = 'w',
        default_value_t = 400,
        name = "low watts",
        help = "Number of Watts for setting a low cap"
    )]
    pub cap_low_watts: u64,

    #[arg(
        long = "cap_high",
        short = 'W',
        default_value_t = 580,
        name = "high watts",
        help = "Number of Watts for setting a high cap, used before setting a low cap"
    )]
    pub cap_high_watts: u64,

    #[arg(
        long,
        short,
        default_value = "./stats",
        name = "stats directory",
        help = "Directory to store runtime stats in"
    )]
    pub stats_dir: String,

    #[arg(
        long,
        default_value = "/home_nfs/wainj/local/bin/firestarter",
        name = "firestarter path",
        help = "Path to firestarter executable (relative or absolute)"
    )]
    pub firestarter: String,

    #[arg(
        long,
        default_value = "/usr/bin/ipmitool",
        name = "ipmi path",
        help = "Path to ipmi executable (relative or absolute)"
    )]
    pub ipmi: String,
}
