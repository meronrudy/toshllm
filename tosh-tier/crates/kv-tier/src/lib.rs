#![forbid(unsafe_code)]
#![allow(clippy::must_use_candidate)]

use tier_core::{ByteSize, LayerId, Tier};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KvBlock {
    pub sequence_id: u64,
    pub layer: LayerId,
    pub token_start: u64,
    pub token_count: u32,
    pub bytes: ByteSize,
    pub tier: Tier,
    pub generation: u64,
}

impl KvBlock {
    pub fn token_end(self) -> u64 {
        self.token_start.saturating_add(u64::from(self.token_count))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KvPolicy {
    pub recent_window_tokens: u64,
    pub host_spill_enabled: bool,
}

impl Default for KvPolicy {
    fn default() -> Self {
        Self {
            recent_window_tokens: 16_384,
            host_spill_enabled: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KvActionKind {
    Keep,
    SpillToHost,
    RestoreToVram,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KvAction {
    pub sequence_id: u64,
    pub layer: LayerId,
    pub token_start: u64,
    pub kind: KvActionKind,
}

pub fn classify(block: KvBlock, current_token: u64, policy: KvPolicy) -> KvActionKind {
    if !policy.host_spill_enabled {
        return KvActionKind::Keep;
    }
    let hot_floor = current_token.saturating_sub(policy.recent_window_tokens);
    match block.tier {
        Tier::Vram(_) if block.token_end() <= hot_floor => KvActionKind::SpillToHost,
        Tier::HostRam if block.token_end() > hot_floor => KvActionKind::RestoreToVram,
        _ => KvActionKind::Keep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tier_core::GpuId;

    #[test]
    fn cold_vram_block_is_spill_candidate() {
        let block = KvBlock {
            sequence_id: 1,
            layer: LayerId(0),
            token_start: 0,
            token_count: 1024,
            bytes: ByteSize(1),
            tier: Tier::Vram(GpuId(0)),
            generation: 0,
        };
        let policy = KvPolicy {
            recent_window_tokens: 2048,
            host_spill_enabled: true,
        };
        assert_eq!(classify(block, 8192, policy), KvActionKind::SpillToHost);
    }
}
