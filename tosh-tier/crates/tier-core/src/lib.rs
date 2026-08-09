#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Tier {
    Disk,
    HostRam,
    Vram(u16),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TensorId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ExpertId(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LayerId(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ByteSize(pub u64);
