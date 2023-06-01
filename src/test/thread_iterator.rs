use enum_iterator::{all, All};
use itertools::{iproduct, ConsTuples, Itertools, Permutations, Product};
use std::vec;

use crate::test::{POWER_HIGH, POWER_LOW, CappingOrder, Operation, CapStep, Test};

pub struct ThreadTestSuite {
    pub iter: OrderOperationStepPowerThreadsIter,
}

impl ThreadTestSuite {
    pub fn new(online_cores: u64) -> Self {
        let n_threads: Vec<u64> = (0..11).map(|t| online_cores - t).collect();
        Self {
            iter: iproduct!(
                all::<CappingOrder>(),
                all::<Operation>(),
                all::<CapStep>(),
                vec![POWER_LOW, POWER_HIGH].into_iter().permutations(2),
                n_threads
            ),
        }
    }
}

impl Iterator for ThreadTestSuite {
    type Item = Test;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((capping_order, operation, step, power_levels, n_threads)) =
            self.iter.next() {
                return Some(Self::Item {
                    capping_order,
                    operation,
                    step,
                    cap_from: power_levels[0],
                    cap_to: power_levels[1],
                    load_pct: 0,
                    load_period: 0,
                    n_threads
                }
            );
        }
        None
    }
}

type OrderOperationStepTuple = ((CappingOrder, Operation), CapStep);
type OrderOperationStepPowerTuple = ((CappingOrder, Operation, CapStep), Vec<u64>);
type OrderOperationStepPowerThreadsTuple = ((CappingOrder, Operation, CapStep, Vec<u64>), u64);


type PowerPermutations = Permutations<IterU64>;
type IterU64 = vec::IntoIter<u64>;

type OrderOperation = Product<All<CappingOrder>, All<Operation>>;
type OrderOperationStep = Product<OrderOperation, All<CapStep>>;
type OrderOperationStepPower = Product<OrderOperationStepIter, PowerPermutations>;
type OrderOperationStepPowerThreads = Product<OrderOperationStepPowerIter, IterU64>;

type OrderOperationStepIter = ConsTuples<OrderOperationStep, OrderOperationStepTuple>;
type OrderOperationStepPowerIter = ConsTuples<OrderOperationStepPower, OrderOperationStepPowerTuple>;
type OrderOperationStepPowerThreadsIter = ConsTuples<OrderOperationStepPowerThreads, OrderOperationStepPowerThreadsTuple>;

