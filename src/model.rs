use chrono::{DateTime, Utc, serde::ts_seconds_option};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Firestarter {
    pub runtime_secs: u64,
    pub load_pct: Option<u64>,
    pub load_period: Option<u64>,
    pub n_threads: Option<u64>,
}

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
    #[serde(with = "ts_seconds_option")]
    timestamp: Option<DateTime<Utc>>,
    data: Vec<RaplData>,
}
