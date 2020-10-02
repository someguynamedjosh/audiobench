#[cfg(not(feature = "no-trivial"))]
mod parse_native_data;

#[cfg(not(feature = "no-trivial"))]
pub use parse_native_data::*;
