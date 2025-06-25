//! Wasmerang - A source-based routing WASM filter for Envoy
//!
//! This crate provides a TCP WASM filter that routes traffic based on the source IP address.
//! The core routing logic is separated from the WASM-specific implementation for better
//! testability and reusability.

pub mod core;

#[cfg(target_arch = "wasm32")]
pub mod wasm_filter;

// Re-export core types for easier access
pub use core::{Config, Router};

// The protobuf module (generated code)
pub mod set_envoy_filter_state {
    include!("generated/envoy.source.extensions.common.wasm.rs");
}
