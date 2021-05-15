#![no_std]

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, FromPrimitive)]
pub enum Ty {
    StartInitialization,
    AddFrames,
    EndInitialization,
    Allocate,
    Free,
}
