
pub use pointer::{PinnedPointer,Pointer, RawPointer};
pub use collector::MemoryInterface;
pub use object::Traceable; 

mod collector;
mod object;
mod pointer;