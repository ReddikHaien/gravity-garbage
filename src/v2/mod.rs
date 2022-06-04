pub use collector::MemoryInterface;
pub use object::Traceable;
pub use pointer::{PinnedPointer, Pointer, RawPointer};

mod collector;
mod object;
mod pointer;
