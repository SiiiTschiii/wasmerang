use log::{info};
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_stream_context(|context_id, _| -> Box<dyn StreamContext> {
        Box::new(DestIpLogger { context_id })
    });
}

struct DestIpLogger {
    context_id: u32,
}

impl Context for DestIpLogger {}

impl StreamContext for DestIpLogger {
    // See https://github.com/proxy-wasm/proxy-wasm-rust-sdk/blob/main/src/traits.rs#L259
    fn on_new_connection(&mut self) -> Action {
        info!("[TCP WASM] New connection established (context #{})", self.context_id);
        // Log source address
        // Property names based on the Proxy-Wasm ABI version 0.2.1
        // https://github.com/proxy-wasm/spec/tree/main/abi-versions/v0.2.1#downstream-connection-properties
        if let Some(val) = self.get_property(vec!["source", "address"]) {
            if let Ok(s) = String::from_utf8(val) {
                info!("[TCP WASM] Source address: {}", s);
            } else {
                info!("[TCP WASM] Source address: (non-UTF8)");
            }
        } else {
            info!("[TCP WASM] Source address not found");
        }
        // Log destination address
        if let Some(val) = self.get_property(vec!["destination", "address"]) {
            if let Ok(s) = String::from_utf8(val) {
                info!("[TCP WASM] Destination address: {}", s);
            } else {
                info!("[TCP WASM] Destination address: (non-UTF8)");
            }
        } else {
            info!("[TCP WASM] Destination address not found");
        }
        Action::Continue
    }
}
