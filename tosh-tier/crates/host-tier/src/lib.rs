#![forbid(unsafe_code)]
#![allow(clippy::must_use_candidate)]

use tier_core::{ByteSize, TensorId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VmAdvice {
    Normal,
    Sequential,
    Random,
    WillNeed,
    DontNeed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MappedRegionSpec {
    pub tensor: TensorId,
    pub file_offset: u64,
    pub length: ByteSize,
    pub advice: VmAdvice,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostBudget {
    pub physical_memory: ByteSize,
    pub process_limit: ByteSize,
    pub operating_system_reserve: ByteSize,
}

impl HostBudget {
    pub fn usable(self) -> ByteSize {
        let physical_usable = self
            .physical_memory
            .saturating_sub(self.operating_system_reserve);
        ByteSize(physical_usable.0.min(self.process_limit.0))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryPressure {
    Normal,
    Warning,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostTierDecision {
    pub mapped: bool,
    pub keep_resident: bool,
    pub advice: VmAdvice,
}

pub fn decision_for_pressure(pressure: MemoryPressure, hot: bool) -> HostTierDecision {
    match (pressure, hot) {
        (MemoryPressure::Critical, false) => HostTierDecision {
            mapped: true,
            keep_resident: false,
            advice: VmAdvice::DontNeed,
        },
        (_, true) => HostTierDecision {
            mapped: true,
            keep_resident: true,
            advice: VmAdvice::WillNeed,
        },
        _ => HostTierDecision {
            mapped: true,
            keep_resident: false,
            advice: VmAdvice::Random,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_budget_preserves_os_reserve() {
        let budget = HostBudget {
            physical_memory: ByteSize(100),
            process_limit: ByteSize(90),
            operating_system_reserve: ByteSize(20),
        };
        assert_eq!(budget.usable(), ByteSize(80));
    }
}
