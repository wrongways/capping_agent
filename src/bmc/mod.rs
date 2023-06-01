pub mod bmc;
pub mod monitor_bmc;
use crate::bmc::bmc::BMC_CapSetting;

use chrono::{DateTime, Utc};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct BMCStats {
    pub timestamp: DateTime<Utc>,
    pub power: u64,
    pub cap_level: u64,
    pub cap_is_active: bool,
}

impl BMCStats {
    pub fn new(power: u64, cap_settings: &BMC_CapSetting) -> Self {
        Self {
            timestamp: Utc::now(),
            power,
            cap_level: cap_settings.power_limit,
            cap_is_active: cap_settings.is_active,
        }
    }
}
