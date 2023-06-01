use crate::bmc::bmc::BMC;
use crate::bmc::BMCStats;
use log::{info, trace};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use crate::CONFIGURATION;

const BMC_INTER_COMMAND_SLEEP_MILLIS: u64 = 500;
const BMC_POLL_INTERVAL_MILLIS: u64 = 500;


/// Periodically polls the BMC for power reading and saves the result. Runs on its own thread.
/// Each time through the loop, checks for a message from the main monitor thread that signals
/// that this thread can exit. Before exiting, saves results to CSV file.
pub fn monitor_bmc(rx: Receiver<()>) -> Vec::<BMCStats> {
    info!("\tBMC: launched");

    let mut stats = Vec::<BMCStats>::new();
    let bmc = BMC::new(
        &CONFIGURATION.bmc_hostname,
        &CONFIGURATION.bmc_username,
        &CONFIGURATION.bmc_password,
        &CONFIGURATION.ipmi
    );

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
        let reading = BMCStats::new(current_power, &current_cap_settings);

        trace!("BMC power reading: {reading:#?}");
        stats.push(reading);

        thread::sleep(Duration::from_millis(BMC_POLL_INTERVAL_MILLIS));
    }


    info!("\tBMC: Exiting");
    stats
}
