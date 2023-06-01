pub mod load_iterator;
pub mod thread_iterator;

use enum_iterator::Sequence;

pub const LOAD_PERIODS_US: [u64; 2] = [10_000, 1_000_000];
pub const POWER_HIGH: u64 = 580;
pub const POWER_LOW: u64 = 200;

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


#[derive(Debug, PartialEq, Sequence, Clone)]
pub enum CapStep {
    OneShot,
    Step,
}

#[derive(Debug, PartialEq, Sequence, Clone)]
pub enum CappingOrder {
    LevelBeforeActivate,
    LevelAfterActivate,
    LevelToLevel,
    LevelToLevelActivate,
}

#[derive(Debug, Copy, Clone, PartialEq, Sequence)]
pub enum Operation {
    Activate,
    Deactivate,
}
