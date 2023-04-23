#![feature(allocator_api)]
#![feature(layout_for_ptr)]
#![feature(unsize)]
#![feature(dispatch_from_dyn)]
#![feature(coerce_unsized)]

pub mod mem_block;
pub mod manager;
pub mod traceable;

pub mod prelude{
    pub use crate::manager::*;
    pub use crate::mem_block::*;
    pub use crate::traceable::*;
}