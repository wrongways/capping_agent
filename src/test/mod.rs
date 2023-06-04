pub mod load_iterator;
pub mod thread_iterator;

use crate::Timestamps;
use crate::model::ServerInfo;

use enum_iterator::Sequence;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

pub const LOAD_PERIODS_US: [u64; 2] = [10_000, 1_000_000];
pub const POWER_HIGH: u64 = 580;
pub const POWER_LOW: u64 = 200;

type Timestamp = DateTime<Utc>;

#[derive(Debug)]
pub struct Test {
    pub capping_order: CappingOrder,
    pub operation: Operation,
    pub step: CapStep,
    pub cap_from: u64,
    pub cap_to: u64,
    pub load_pct: u64,
    pub load_period: u64,
    pub n_threads: u64,
}

#[derive(Serialize, Deserialize)]
pub struct TestRun {
    pub start_timestamp: Timestamp,
    pub end_timestamp: Timestamp,
    pub cap_timestamp: Timestamp,
    pub capping_order: CappingOrder,
    pub operation: Operation,
    pub step: CapStep,
    pub cap_from: u64,
    pub cap_to: u64,
    pub load_pct: u64,
    pub load_period: u64,
    pub n_threads: u64,
}

impl TestRun {
    pub fn new(timestamps: Timestamps, test: Test) -> Self {
        Self {
            start_timestamp: timestamps.0,
            cap_timestamp: timestamps.1,
            end_timestamp: timestamps.2,
            capping_order: test.capping_order,
            operation: test.operation,
            step: test.step,
            cap_from: test.cap_from,
            cap_to: test.cap_to,
            load_pct: test.load_pct,
            load_period: test.load_period,
            n_threads: test.n_threads,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuiteInfo {
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
    pub server_info: ServerInfo,
}


#[derive(Debug, PartialEq, Sequence, Clone, Serialize, Deserialize)]
pub enum CapStep {
    OneShot,
    Step,
}

#[derive(Debug, PartialEq, Sequence, Clone, Serialize, Deserialize)]
pub enum CappingOrder {
    LevelBeforeActivate,
    LevelAfterActivate,
    LevelToLevel,
    LevelToLevelActivate,
}

#[derive(Debug, Copy, Clone, PartialEq, Sequence, Serialize, Deserialize)]
pub enum Operation {
    Activate,
    Deactivate,
}
