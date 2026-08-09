#![forbid(unsafe_code)]
#![allow(clippy::must_use_candidate)]

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Tier {
    Disk,
    HostRam,
    Vram(GpuId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GpuId(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TensorId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExpertId(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LayerId(pub u32);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ByteSize(pub u64);

impl ByteSize {
    pub const ZERO: Self = Self(0);

    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyMode {
    Static,
    Lru,
    Lfu,
    Windowed,
    Predictive,
    Adaptive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeBudget {
    pub host_limit: ByteSize,
    pub expert_cache_limit: ByteSize,
    pub kv_cache_limit: ByteSize,
    pub vram_reserve: ByteSize,
}

impl Default for RuntimeBudget {
    fn default() -> Self {
        Self {
            host_limit: ByteSize::ZERO,
            expert_cache_limit: ByteSize::ZERO,
            kv_cache_limit: ByteSize::ZERO,
            vram_reserve: ByteSize::ZERO,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Residency {
    pub canonical: Tier,
    replicas: BTreeSet<Tier>,
}

impl Residency {
    pub fn new(canonical: Tier) -> Self {
        let mut replicas = BTreeSet::new();
        replicas.insert(canonical);
        Self { canonical, replicas }
    }

    pub fn contains(&self, tier: Tier) -> bool {
        self.replicas.contains(&tier)
    }

    pub fn add_replica(&mut self, tier: Tier) {
        self.replicas.insert(tier);
    }

    pub fn remove_replica(&mut self, tier: Tier) {
        if tier != self.canonical {
            self.replicas.remove(&tier);
        }
    }

    pub fn replicas(&self) -> impl Iterator<Item = Tier> + '_ {
        self.replicas.iter().copied()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementReason {
    Demand,
    Prefetch,
    Pressure,
    PolicyRebalance,
    KvWindow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementAction {
    pub tensor: TensorId,
    pub source: Tier,
    pub destination: Tier,
    pub bytes: ByteSize,
    pub reason: PlacementReason,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RuntimeCounters {
    pub router_events: u64,
    pub selected_experts: u64,
    pub transfers_submitted: u64,
    pub transfers_completed: u64,
    pub host_to_device_bytes: u64,
    pub device_to_host_bytes: u64,
    pub peer_bytes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_residency_cannot_be_removed() {
        let mut residency = Residency::new(Tier::HostRam);
        residency.add_replica(Tier::Vram(GpuId(0)));
        residency.remove_replica(Tier::HostRam);
        assert!(residency.contains(Tier::HostRam));
        assert!(residency.contains(Tier::Vram(GpuId(0))));
    }
}
