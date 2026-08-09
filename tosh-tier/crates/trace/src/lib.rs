#![forbid(unsafe_code)]

use tier_core::{ExpertId, LayerId};

pub const TRACE_SCHEMA: &str = "tosh-tier-trace.v1";

#[derive(Clone, Debug, PartialEq)]
pub struct ExpertSelection {
    pub request_id: u64,
    pub sequence_id: u64,
    pub token_position: u64,
    pub layer: LayerId,
    pub experts: Vec<ExpertId>,
}
