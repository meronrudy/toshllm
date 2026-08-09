#![forbid(unsafe_code)]
#![allow(clippy::must_use_candidate, clippy::cast_precision_loss)]

use std::collections::BTreeMap;

use tier_core::{ByteSize, ExpertId, GpuId, LayerId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpertDescriptor {
    pub id: ExpertId,
    pub layer: LayerId,
    pub bytes: ByteSize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExpertStats {
    pub activations: u64,
    pub last_token: u64,
    pub first_token: u64,
}

impl ExpertStats {
    pub fn observe(&mut self, token: u64) {
        if self.activations == 0 {
            self.first_token = token;
        }
        self.activations = self.activations.saturating_add(1);
        self.last_token = token;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CachePolicy {
    Static,
    Lru,
    Lfu,
    Windowed,
    Predictive,
    Adaptive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CacheConfig {
    pub gpu: GpuId,
    pub budget: ByteSize,
    pub minimum_residency_tokens: u64,
    pub reevaluate_every_tokens: u64,
    pub maximum_promotions_per_window: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheActionKind {
    Promote,
    Evict,
    Keep,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CacheAction {
    pub expert: ExpertId,
    pub kind: CacheActionKind,
}

#[derive(Debug, Default)]
pub struct ExpertCacheState {
    stats: BTreeMap<ExpertId, ExpertStats>,
    resident: BTreeMap<ExpertId, u64>,
}

impl ExpertCacheState {
    pub fn observe(&mut self, expert: ExpertId, token: u64) {
        self.stats.entry(expert).or_default().observe(token);
    }

    pub fn mark_resident(&mut self, expert: ExpertId, token: u64) {
        self.resident.insert(expert, token);
    }

    pub fn mark_evicted(&mut self, expert: ExpertId) {
        self.resident.remove(&expert);
    }

    pub fn is_resident(&self, expert: ExpertId) -> bool {
        self.resident.contains_key(&expert)
    }

    pub fn stats(&self, expert: ExpertId) -> Option<ExpertStats> {
        self.stats.get(&expert).copied()
    }

    pub fn rank_lru(&self) -> Vec<ExpertId> {
        let mut ranked: Vec<_> = self.stats.iter().collect();
        ranked.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.last_token));
        ranked.into_iter().map(|(id, _)| *id).collect()
    }

    pub fn rank_lfu(&self) -> Vec<ExpertId> {
        let mut ranked: Vec<_> = self.stats.iter().collect();
        ranked.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.activations));
        ranked.into_iter().map(|(id, _)| *id).collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionScore {
    pub from: ExpertId,
    pub to: ExpertId,
    pub probability: f32,
}

#[derive(Debug, Default)]
pub struct TransitionTable {
    counts: BTreeMap<(ExpertId, ExpertId), u64>,
    totals: BTreeMap<ExpertId, u64>,
}

impl TransitionTable {
    pub fn observe(&mut self, from: ExpertId, to: ExpertId) {
        let count = self.counts.entry((from, to)).or_default();
        *count = count.saturating_add(1);
        let total = self.totals.entry(from).or_default();
        *total = total.saturating_add(1);
    }

    pub fn top_successors(&self, from: ExpertId, limit: usize) -> Vec<TransitionScore> {
        let total = self.totals.get(&from).copied().unwrap_or_default();
        if total == 0 || limit == 0 {
            return Vec::new();
        }
        let mut rows: Vec<_> = self
            .counts
            .iter()
            .filter(|((source, _), _)| *source == from)
            .map(|((_, target), count)| (*target, *count))
            .collect();
        rows.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        rows.truncate(limit);
        rows.into_iter()
            .map(|(to, count)| TransitionScore {
                from,
                to,
                probability: count as f32 / total as f32,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lru_ranks_recent_experts_first() {
        let mut state = ExpertCacheState::default();
        state.observe(ExpertId(1), 1);
        state.observe(ExpertId(2), 9);
        assert_eq!(state.rank_lru(), vec![ExpertId(2), ExpertId(1)]);
    }
}
