//! Look at this [Link](std::iter::Iter)

#[macro_use]
extern crate pest_derive;

pub mod ast;
mod high_level;
#[cfg(not(feature = "no-llvmir"))]
pub mod llvmir;
#[cfg(not(feature = "no-resolved"))]
pub mod resolved;
pub mod shared;
#[cfg(not(feature = "no-trivial"))]
pub mod trivial;
pub mod util;
#[cfg(not(feature = "no-vague"))]
pub mod vague;

pub use high_level::compiler::Compiler;
