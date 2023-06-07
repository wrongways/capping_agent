use enum_iterator::{all, All};
use itertools::{iproduct, ConsTuples, Itertools, Permutations, Product};
use std::vec;

use crate::test::{POWER_HIGH, POWER_LOW, CappingOrder, Operation, CapStep, Test};

impl LoadTestSuite {
    pub fn new() -> Self {
        let loads: Vec<u64> = (0..11).map(|p| 100 - p).collect();
        Self {
            iter: iproduct!(
                all::<CappingOrder>(),
                all::<Operation>(),
                all::<CapStep>(),
                vec![POWER_LOW, POWER_HIGH].into_iter().permutations(2),
                loads,
                vec![10_000, 1_000_000]
            ),
        }
    }
}

impl Default for LoadTestSuite {
    fn default() -> Self {
        LoadTestSuite::new()
    }
}

impl Iterator for LoadTestSuite {
    type Item = Test;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((capping_order, operation, step, power_levels, load_pct, load_period)) =
            self.iter.next()
        {
            return Some(Test {
                capping_order,
                operation,
                step,
                cap_from: power_levels[0],
                cap_to: power_levels[1],
                load_pct,
                load_period,
                n_threads: 0
            });
        }
        None
    }
}

type OrderOperationStepTuple = ((CappingOrder, Operation), CapStep);
type OrderOperationStepPowerTuple = ((CappingOrder, Operation, CapStep), Vec<u64>);
type OrderOperationStepPowerTupleLoadTuple = ((CappingOrder, Operation, CapStep, Vec<u64>), u64);
type OrderOperationStepPowerTupleLoadPeriodTuple = ((CappingOrder, Operation, CapStep, Vec<u64>, u64), u64);

type PowerPermutations = Permutations<IterU64>;
type IterU64 = vec::IntoIter<u64>;

type OrderOperation = Product<All<CappingOrder>, All<Operation>>;
type OrderOperationStep = Product<OrderOperation, All<CapStep>>;
type OrderOperationStepPower = Product<OrderOperationStepIter, PowerPermutations>;
type OrderOperationStepPowerLoad = Product<OrderOperationStepPowerIter, IterU64>;
type OrderOperationStepPowerLoadPeriod = Product<OrderOperationsStepPowerLoadIter, IterU64>;

type OrderOperationStepIter = ConsTuples<OrderOperationStep, OrderOperationStepTuple>;
type OrderOperationStepPowerIter = ConsTuples<OrderOperationStepPower, OrderOperationStepPowerTuple>;
type OrderOperationsStepPowerLoadIter = ConsTuples<OrderOperationStepPowerLoad, OrderOperationStepPowerTupleLoadTuple>;
type OrderOperationsStepPowerLoadPeriodIter = ConsTuples<OrderOperationStepPowerLoadPeriod, OrderOperationStepPowerTupleLoadPeriodTuple>;


pub struct LoadTestSuite {
    pub iter: OrderOperationsStepPowerLoadPeriodIter,
}

