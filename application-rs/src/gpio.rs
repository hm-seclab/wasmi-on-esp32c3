use core::marker::PhantomData;

use embedded_hal::digital::v2::{InputPin, OutputPin};

use crate::{
    error::WasmError,
    gpio_mode::{GpioPin, Input, Output, Unknown},
    runtime,
};

macro_rules! impl_gpio_trait {
    ($trait:ty, $type:ty) => {
        impl $trait for $type {
            fn pin(&self) -> u32 {
                self.pin
            }

            fn port(&self) -> u32 {
                self.port
            }
        }
    };
}

/// A pin that is generic over its direction (input or output).
pub struct Pin<MODE> {
    port: u32,
    pin: u32,
    _mode: PhantomData<MODE>,
}

impl_gpio_trait!(GpioPin, Pin<Input>);
impl_gpio_trait!(GpioPin, Pin<Output>);

impl Pin<Unknown> {
    /// Create a new pin with a given pin and port.
    pub fn new(port: u32, pin: u32,) -> Self {
        Self {
            pin,
            port,
            _mode: PhantomData,
        }
    }

    /// Converts a pin into an input pin. This operation is irreversible.
    pub fn into_input(self) -> Result<Pin<Input>, WasmError> {
        check_error!(unsafe { runtime::gpio_init(self.port, self.pin, true) });

        Ok(Pin {
            pin: self.pin,
            port: self.port,
            _mode: PhantomData,
        })
    }

    /// Converts a pin into an output pin. This operation is irreversible.
    pub fn into_output(self) -> Result<Pin<Output>, WasmError> {
        check_error!(unsafe { runtime::gpio_init(self.port, self.pin, false) });

        Ok(Pin {
            pin: self.pin,
            port: self.port,
            _mode: PhantomData,
        })
    }
}

impl InputPin for Pin<Input> {
    type Error = WasmError;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let mut gpio_state: u32 = 0;
        check_error!(unsafe {
            runtime::gpio_read(self.port(), self.pin(), &mut gpio_state as *mut u32)
        });

        Ok(gpio_state == 1)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let mut gpio_state: u32 = 0;
        check_error!(unsafe {
            runtime::gpio_read(self.port(), self.pin(), &mut gpio_state as *mut u32)
        });

        Ok(gpio_state == 0)
    }
}

impl OutputPin for Pin<Output> {
    type Error = WasmError;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        check_error!(unsafe { runtime::gpio_write(self.port(), self.pin(), 0) });

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        check_error!(unsafe { runtime::gpio_write(self.port(), self.pin(), 1) });

        Ok(())
    }
}
