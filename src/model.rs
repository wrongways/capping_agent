use chrono::{DateTime, Utc, serde::ts_milliseconds_option};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::fmt;

pub type Semaphore = Arc<RwLock<bool>>;
pub fn is_running() -> Semaphore {
    let semaphore: bool = false;
    Arc::new(RwLock::new(semaphore))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub model: String,
    pub os: String,
    pub manufacturer: String,
    pub cpu_version: String,
    pub online_cpus: u64,
    pub min_mhz: u64,
    pub max_mhz: u64,
    pub threads_per_core: u64,
    pub cores_per_socket: u64,
    pub n_sockets: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiosInfo {
    pub vendor: String,
    pub version: String,
    pub revision: String,
    pub release_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    pub system_info: SystemInfo,
    pub bios_info: BiosInfo,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RaplData {
    pub domain: String,
    pub power_watts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RaplRecord {
    #[serde(with = "ts_milliseconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub data: Vec<RaplData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirestarterParams {
    pub runtime_secs: u64,
    pub load_pct: u64,
    pub load_period_us: u64,
    pub n_threads: u64,
}

impl fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "          hostname: {}\n\
                          model: {}\n\
                             os: {}\n\
                   manufacturer: {}\n\
                            cpu: {}\n\
                      cpu count: {}\n\
                 min freq (MHz): {}\n\
                 max freq (MHz): {}\n\
               threads per core: {}\n\
               cores per socket: {}\n\
                   socket count: {}\n",
            &self.hostname,
            &self.model,
            &self.os,
            &self.manufacturer,
            &self.cpu_version,
            &self.online_cpus,
            &self.min_mhz,
            &self.max_mhz,
            &self.threads_per_core,
            &self.cores_per_socket,
            &self.n_sockets
        )
    }
}

impl fmt::Display for BiosInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
        "            vendor: {}\n\
                    version: {}\n\
                   revision: {}\n\
               release date: {}\n",
            &self.vendor,
            &self.version,
            &self.revision,
            &self.release_date
        )
    }
}
