//! Contains widgets that make up the top-level structure of the GUI, I.E. tabs and header.

pub mod graph;
mod header;
mod patch_browser;

pub use header::Header;
pub use patch_browser::*;