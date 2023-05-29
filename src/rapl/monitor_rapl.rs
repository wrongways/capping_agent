use crate::rapl::rapl::{RAPL_Readings, RAPL_Reading, RAPL};
use crate::ResultType;
use crate::model::{RaplData, RaplRecord};

use chrono::SecondsFormat;
use log::debug;
use log::{info, trace};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

const POLL_FREQ_HZ: u64 = 2;

/// Periodically reads all the `energy_uj` files and saves the result. Runs on its own thread.
/// Each time through the loop, checks for a message from the main monitor thread that signals
/// that this thread can exit. Before exiting, saves results to CSV file.
pub fn monitor_rapl(rx: &Receiver<()>) {
    info!("\tRAPL: launched");

    let mut stats = Vec::<RAPL_Readings>::new();
    let rapl = RAPL::new();
    let sleep_millis = 1000 / POLL_FREQ_HZ;
    loop {
        if rx.try_recv().is_ok() {
            trace!("\tRAPL: got message - exiting");
            break;
        }
        let energy_reading = rapl.read_current_energy();
        trace!("{energy_reading}");
        stats.push(energy_reading);
        thread::sleep(Duration::from_millis(sleep_millis));
    }
    convert_energy_to_power(stats)
}



/// Does what it says on the packet - divides energy deltas by time deltas to give power.
fn convert_energy_to_power(stats: &[RAPL_Readings]) -> Vec<RaplRecord> {
    // The units of reading are µJ

    let mut readings = Vec::with_capacity(stats.len());
    // sanity check: ensure all reading have same # entries
    let n_domains = stats[0].readings.len();

    // for stats[1...], calculate power by calculating the
    // energy change from the previous reading and dividing by
    // the time delta for each RAPL domain. The total power is
    // the sum of the domains.

    // need to check for wrap-around - keep tabs on max_energy_uj and previous reading
    let max_energy_uj = RAPL::max_energy();

    // By using skip(1), the index from the enumerate is one behind the
    // current row, i.e. it points to the preceding row, which is exactly
    // what's needed to calculate the deltas.
    for (stat_index, stat) in stats.iter().skip(1).enumerate() {
        assert_eq!(stat.readings.len(), n_domains);
        let mut power_readings: Vec<RaplData> = Vec::with_capacity(n_domains);
        let time_delta = stat.timestamp - stats[stat_index].timestamp;
        let time_midpoint = stat.timestamp - (time_delta / 2);

        // Loop over the domains
        for (domain_index, reading) in stat.readings.iter().enumerate() {
            let previous_reading = stats[stat_index].readings[domain_index].reading;
            let current_reading = reading.reading;

            // check for wrap-around
            let energy_delta_uj = {
                if current_reading < previous_reading {
                    max_energy_uj - previous_reading + current_reading // wrapped
                } else {
                    current_reading - previous_reading // no wrap
                }
            };

            // time delta is always positive so no loss of sign - and in any case makes no difference
            #[allow(clippy::cast_sign_loss)]
            let power_watts = energy_delta_uj / time_delta.num_milliseconds() as u64;
            power_readings.push(RAPL_Reading {
                domain: reading.domain.clone(),
                power_watts,
            });
        }
        let datapoint = RaplRecord {timestamp: time_midpoint, data: power_readings};
        readings.push(datapoint);
    }
    readings
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;


    #[test]
    fn test_energy_to_power() {
        let r1 = RAPL_Reading {domain: String::from("0"), reading: 0};
        let r2 = RAPL_Reading {domain: String::from("1"), reading: 0};
        let r3 = RAPL_Reading {domain: String::from("0"), reading: 100_000_000};
        let r4 = RAPL_Reading {domain: String::from("1"), reading:  50_000_000};
        let r5 = RAPL_Reading {domain: String::from("0"), reading: 200_000_000};
        let r6 = RAPL_Reading {domain: String::from("1"), reading: 100_000_000};
        let r7 = RAPL_Reading {domain: String::from("0"), reading: 200_000_000};
        let r8 = RAPL_Reading {domain: String::from("1"), reading: 100_000_000};
        let r9 = RAPL_Reading {domain: String::from("0"), reading: 400_000_000};
        let r10 = RAPL_Reading {domain: String::from("1"), reading: 200_000_000};

        let t0 = Utc::now();
        let t1 = t0 + chrono::Duration::milliseconds(1000);
        let t2 = t0 + chrono::Duration::milliseconds(2000);
        let t3 = t0 + chrono::Duration::milliseconds(3000);
        let t4 = t0 + chrono::Duration::milliseconds(5000);

        let readings1 = RAPL_Readings{timestamp: t0, readings: vec![r1, r2]};
        let readings2 = RAPL_Readings{timestamp: t1, readings: vec![r3, r4]};
        let readings3 = RAPL_Readings{timestamp: t2, readings: vec![r5, r6]};
        let readings4 = RAPL_Readings{timestamp: t3, readings: vec![r7, r8]};
        let readings5 = RAPL_Readings{timestamp: t4, readings: vec![r9, r10]};

        let energy_stats = vec![readings1, readings2, readings3, readings4, readings5];
        let power_stats = convert_energy_to_power(&energy_stats);

        assert_eq!(power_stats.len(), energy_stats.len() - 1);

        // check power
        assert_eq!(power_stats[0].readings[0].reading, 100);
        assert_eq!(power_stats[0].readings[1].reading,  50);
        assert_eq!(power_stats[1].readings[0].reading, 100);
        assert_eq!(power_stats[1].readings[1].reading,  50);
        assert_eq!(power_stats[2].readings[0].reading,   0);
        assert_eq!(power_stats[2].readings[1].reading,   0);
        assert_eq!(power_stats[3].readings[0].reading, 100);
        assert_eq!(power_stats[3].readings[1].reading,  50);

        // check timestamps
        assert_eq!(power_stats[0].timestamp, t0 + chrono::Duration::milliseconds(500));
        assert_eq!(power_stats[1].timestamp, t0 + chrono::Duration::milliseconds(1500));
        assert_eq!(power_stats[2].timestamp, t0 + chrono::Duration::milliseconds(2500));
        assert_eq!(power_stats[3].timestamp, t0 + chrono::Duration::milliseconds(4000));
    }
}
