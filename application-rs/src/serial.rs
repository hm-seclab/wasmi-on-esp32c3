use core::fmt::Write as CoreWrite;
use embedded_hal::serial::{Read, Write};
use nb::block;

use crate::{
    error::WasmError,
    gpio::Pin,
    gpio_mode::{GpioPin, Input, Output},
    runtime,
};

type Word = u8;

/// Pins related to the UART protocol. CTS and RTS are
/// optional and don't always need to be present.
pub struct Pins {
    pub rx: Pin<Input>,
    pub tx: Pin<Output>,
    pub cts: Option<Pin<Input>>,
    pub rts: Option<Pin<Output>>,
}

/// Represents a UART connection, holding a handle that's given out when the
/// connection gets registered.
pub struct Uart {
    handle: u8,
}

impl Uart {
    /// Create an instance and register the connection by calling [`runtime::uart_init`].
    /// This could result in an error with an error code specified by the runtime.
    pub fn new(pins: Pins) -> Result<Self, WasmError> {
        let tx = pins.tx;
        let rx = pins.rx;
        let cts = pins
            .cts
            .map_or((core::ptr::null(), core::ptr::null()), |pin| {
                (&pin.port(), &pin.pin())
            });
        let rts = pins
            .rts
            .map_or((core::ptr::null(), core::ptr::null()), |pin| {
                (&pin.port(), &pin.pin())
            });

        let mut handle = 0_u8;
        check_error!(unsafe {
            runtime::uart_init(
                &mut handle as *mut _,
                tx.port(),
                tx.pin(),
                rx.port(),
                rx.pin(),
                cts.0,
                cts.1,
                rts.0,
                rts.1,
            )
        });

        Ok(Self { handle })
    }
}

impl Write<Word> for Uart {
    type Error = WasmError;

    fn write(&mut self, word: Word) -> nb::Result<(), Self::Error> {
        check_nb_error!(unsafe { runtime::uart_write(self.handle, word) });

        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

impl Read<Word> for Uart {
    type Error = WasmError;

    fn read(&mut self) -> nb::Result<Word, Self::Error> {
        let mut value = 0;
        check_nb_error!(unsafe { runtime::uart_read(self.handle, &mut value as *mut u8) });

        Ok(value)
    }
}

impl CoreWrite for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let _ = s.as_bytes().iter().map(|c| block!(self.write(*c))).last();
        Ok(())
    }
}
