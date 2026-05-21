// lib.rs
pub mod ai;
pub mod config;
mod errors;
pub mod git;
pub mod gmsg;
pub mod tui;

pub use gmsg::Gmsg;
#[cfg(test)]
pub mod test_utils;
