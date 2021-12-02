use embedded_hal::blocking::delay::DelayMs;

use crate::runtime;

/// A struct that implements embedded_hal's DelayMs.
pub struct WasmDelay;

impl DelayMs<u32> for WasmDelay {
    fn delay_ms(&mut self, ms: u32) {
        // currently ignoring result
        unsafe { runtime::delay_ms(ms) };
    }
}
