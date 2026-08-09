#![forbid(unsafe_code)]

use tier_core::{ByteSize, ExpertId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpertDescriptor {
    pub id: ExpertId,
    pub bytes: ByteSize,
}
