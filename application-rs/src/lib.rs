#![no_std]

use delay::WasmDelay;
use embedded_hal::blocking::delay::DelayMs;
use gpio::Pin;
use gpio_mode::Unknown;

macro_rules! check_error {
    ($ub:expr) => {
        let res = $ub;
        if res != 0 {
            return Err(WasmError::RuntimeError(res));
        }
    };
}

macro_rules! check_nb_error {
    ($ub:expr) => {
        let res = $ub;
        if res != 0 {
            return Err(nb::Error::Other(WasmError::RuntimeError(res)));
        }
    };
}

pub mod delay;
pub mod error;
pub mod gpio;
pub mod print;
mod runtime;
pub mod serial;

/// A struct for accessing the hardware, as in most hal implementations
/// this struct is hand out as a singleton.
pub struct Periphals;

/// A singleton Periphal.
static mut PERIPHALS: Option<Periphals> = Some(Periphals);

impl Periphals {
    /// Get an instance.
    pub fn take() -> Option<Self> {
        // TODO: wouldn't work if multiple processes are interacting here
        unsafe { PERIPHALS.take() }
    }

    /// Initialize a Gpio pin.
    pub fn init_gpio(&self, port: u32, pin: u32) -> Pin<Unknown> {
        Pin::new(port, pin)
    }
}

impl DelayMs<u32> for Periphals {
    fn delay_ms(&mut self, ms: u32) {
        WasmDelay::delay_ms(&mut WasmDelay, ms);
    }
}

pub mod gpio_mode {
    /// A trait specifying a pin with a port and a pin number.
    pub trait GpioPin {
        fn pin(&self) -> u32;

        fn port(&self) -> u32;
    }

    /// A type that specifies an input direction.
    pub struct Input;

    /// A type that specifies an output direction.
    pub struct Output;

    /// A type that specifies an unknown (not yet decided) direction.
    pub struct Unknown;
}
