//! # IceTray core crate
//!
//! `core` contains all of the low-level IceTray framework details,
//! including serialization, frames, modules, and how to run them.

mod frame;
pub use crate::frame::*;
mod file;
pub use crate::file::*;
mod module;
pub use crate::module::*;
mod tray;
pub use crate::tray::*;