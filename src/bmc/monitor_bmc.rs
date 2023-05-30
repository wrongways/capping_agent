use crate::bmc::bmc::{BMC, BMC_CapSetting};
use log::{info, trace};
use std::fmt;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use chrono::{DateTime, Utc, SecondsFormat};

const BMC_INTER_COMMAND_SLEEP_MILLIS: u64 = 500;
const BMC_POLL_INTERVAL_MILLIS: u64 = 500;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct BMC_Stats {
    pub timestamp: DateTime<Utc>,
    pub power: u64,
    pub cap_level: u64,
    pub cap_is_active: bool,
}

impl fmt::Display for BMC_Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.timestamp.to_rfc3339_opts(SecondsFormat::Millis, false),
            self.power,
            self.cap_level,
            self.cap_is_active,
        )
    }
}

impl BMC_Stats {
    pub fn new(power: u64, cap_settings: &BMC_CapSetting) -> Self {
        Self {
            timestamp: Utc::now(),
            power,
            cap_level: cap_settings.power_limit,
            cap_is_active: cap_settings.is_active,
        }
    }
}

/// Periodically polls the BMC for power reading and saves the result. Runs on its own thread.
/// Each time through the loop, checks for a message from the main monitor thread that signals
/// that this thread can exit. Before exiting, saves results to CSV file.
pub fn monitor_bmc(
    bmc_hostname: &str,
    bmc_username: &str,
    bmc_password: &str,
    ipmi: &str,
    rx: Receiver<()>) -> Vec::<BMC_Stats> {
    info!("\tBMC: launched");

    let mut stats = Vec::<BMC_Stats>::new();
    let bmc = BMC::new(&bmc_hostname, &bmc_username, &bmc_password, &ipmi);
    loop {
        // Check if monitor master asked us to exit with a message on the channel
        if rx.try_recv().is_ok() {
            trace!("\tBMC: got message - exiting");
            break;
        }

        // No message, read current power and capping status
        let current_power = bmc.current_power();
        thread::sleep(Duration::from_millis(BMC_INTER_COMMAND_SLEEP_MILLIS));
        let current_cap_settings = bmc.current_cap_settings();
        let reading = BMC_Stats::new(current_power, &current_cap_settings);

        trace!("BMC power reading: {reading:#?}");
        stats.push(reading);

        thread::sleep(Duration::from_millis(BMC_POLL_INTERVAL_MILLIS));
    }


    info!("\tBMC: Exiting");
    stats
}
