//! Core routing logic for the Wasmerang filter
//!
//! This module contains all the pure Rust logic for configuration parsing,
//! IP address handling, and routing decisions. It has no dependencies on
//! WASM or Envoy-specific code, making it easy to test and reuse.

mod config;
mod router;
mod utils;

pub use config::Config;
pub use router::Router;
pub use utils::extract_last_octet;
