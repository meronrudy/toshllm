#![forbid(unsafe_code)]

use tier_core::{ByteSize, Tier};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransferIntent {
    pub src: Tier,
    pub dst: Tier,
    pub bytes: ByteSize,
}
