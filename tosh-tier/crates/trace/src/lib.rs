#![forbid(unsafe_code)]
#![allow(clippy::must_use_candidate)]

use tier_core::{ByteSize, ExpertId, GpuId, LayerId, Tier};

pub const TRACE_SCHEMA: &str = "tosh-tier-trace.v1";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExpertChoice {
    pub expert: ExpertId,
    pub rank: u16,
    pub weight: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RouterSelection {
    pub request_id: u64,
    pub sequence_id: u64,
    pub token_position: u64,
    pub layer: LayerId,
    pub experts: Vec<ExpertChoice>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VramSample {
    pub gpu: GpuId,
    pub used: ByteSize,
    pub free: ByteSize,
    pub kv_bytes: ByteSize,
    pub expert_bytes: ByteSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransferSample {
    pub transfer_id: u64,
    pub source: Tier,
    pub destination: Tier,
    pub bytes: ByteSize,
    pub start_ns: u64,
    pub complete_ns: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TraceEvent {
    Router(RouterSelection),
    Vram(VramSample),
    Transfer(TransferSample),
    Dropped { count: u64 },
}

pub trait TraceSink {
    fn record(&mut self, event: TraceEvent);
}

#[derive(Default)]
pub struct NullTrace;

impl TraceSink for NullTrace {
    fn record(&mut self, _event: TraceEvent) {}
}

#[derive(Debug, Default)]
pub struct VecTrace {
    events: Vec<TraceEvent>,
    dropped: u64,
    maximum_events: usize,
}

impl VecTrace {
    pub fn bounded(maximum_events: usize) -> Self {
        Self {
            events: Vec::new(),
            dropped: 0,
            maximum_events,
        }
    }

    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

impl TraceSink for VecTrace {
    fn record(&mut self, event: TraceEvent) {
        if self.events.len() >= self.maximum_events {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        self.events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_trace_drops_without_blocking() {
        let mut trace = VecTrace::bounded(1);
        trace.record(TraceEvent::Dropped { count: 0 });
        trace.record(TraceEvent::Dropped { count: 1 });
        assert_eq!(trace.events().len(), 1);
        assert_eq!(trace.dropped(), 1);
    }
}
