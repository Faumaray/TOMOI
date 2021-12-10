/// Struct representing a branch in the [`HuffTree`][crate::tree::HuffTree] struct. 
pub mod branch;
/// Struct representing a [`HuffBranch`'s][crate::tree::branch::HuffBranch] data.
pub mod leaf;
/// Traits signyfing that a type can be stored in a [`HuffTree`][crate::tree::HuffTree] as a letter.
pub mod letter;

mod branch_heap;
mod tree_inner;

pub use tree_inner::{
    HuffTree,
    FromBinError
};
