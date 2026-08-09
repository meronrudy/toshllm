#![forbid(unsafe_code)]

use tier_core::Tier;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KvBlockLocation {
    pub sequence_id: u64,
    pub token_start: u64,
    pub token_count: u32,
    pub tier: Tier,
}
