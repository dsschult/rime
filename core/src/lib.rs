//! # IceTray core crate
//!
//! `core` contains all of the low-level IceTray framework details,
//! including serialization, frames, modules, and how to run them.

mod i3frame;
pub use crate::i3frame::*;
mod i3file;
pub use crate::i3file::*;