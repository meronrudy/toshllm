#![forbid(unsafe_code)]
#![allow(clippy::must_use_candidate)]

use std::collections::VecDeque;

use tier_core::{ByteSize, TensorId, Tier};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferPriority {
    Demand,
    Prefetch,
    Background,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportKind {
    HostStage,
    EventHandoff,
    PeerCopy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransferRequest {
    pub id: u64,
    pub tensor: TensorId,
    pub source: Tier,
    pub destination: Tier,
    pub bytes: ByteSize,
    pub priority: TransferPriority,
    pub transport: TransportKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueError {
    Full,
}

#[derive(Debug)]
pub struct TransferQueue {
    demand: VecDeque<TransferRequest>,
    prefetch: VecDeque<TransferRequest>,
    background: VecDeque<TransferRequest>,
    maximum_depth: usize,
}

impl TransferQueue {
    pub fn new(maximum_depth: usize) -> Self {
        Self {
            demand: VecDeque::new(),
            prefetch: VecDeque::new(),
            background: VecDeque::new(),
            maximum_depth,
        }
    }

    pub fn len(&self) -> usize {
        self.demand.len() + self.prefetch.len() + self.background.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, request: TransferRequest) -> Result<(), QueueError> {
        if self.len() >= self.maximum_depth {
            return Err(QueueError::Full);
        }
        match request.priority {
            TransferPriority::Demand => self.demand.push_back(request),
            TransferPriority::Prefetch => self.prefetch.push_back(request),
            TransferPriority::Background => self.background.push_back(request),
        }
        Ok(())
    }

    pub fn pop_next(&mut self) -> Option<TransferRequest> {
        self.demand
            .pop_front()
            .or_else(|| self.prefetch.pop_front())
            .or_else(|| self.background.pop_front())
    }

    pub fn drop_prefetch(&mut self) -> usize {
        let dropped = self.prefetch.len();
        self.prefetch.clear();
        dropped
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LinkCost {
    pub transport: TransportKind,
    pub latency_ns: u64,
    pub bytes_per_second: u64,
}

impl LinkCost {
    pub fn estimated_ns(self, bytes: ByteSize) -> u64 {
        if self.bytes_per_second == 0 {
            return u64::MAX;
        }
        let transfer_ns = bytes
            .0
            .saturating_mul(1_000_000_000)
            .saturating_div(self.bytes_per_second);
        self.latency_ns.saturating_add(transfer_ns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demand_preempts_prefetch() {
        let mut queue = TransferQueue::new(4);
        let prefetch = TransferRequest {
            id: 1,
            tensor: TensorId(1),
            source: Tier::HostRam,
            destination: Tier::HostRam,
            bytes: ByteSize(1),
            priority: TransferPriority::Prefetch,
            transport: TransportKind::HostStage,
        };
        let demand = TransferRequest {
            id: 2,
            priority: TransferPriority::Demand,
            ..prefetch
        };
        assert!(queue.push(prefetch).is_ok());
        assert!(queue.push(demand).is_ok());
        assert_eq!(queue.pop_next().map(|item| item.id), Some(2));
    }
}
