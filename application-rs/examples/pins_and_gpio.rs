#![no_main]
#![no_std]

use core::fmt::Write;
use core::panic::PanicInfo;

use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    prelude::_embedded_hal_blocking_delay_DelayMs,
};
use wasm_embedded_hal::print;
use wasm_embedded_hal::{
    serial::{Pins, Uart},
    Periphals,
};

#[no_mangle]
fn start() -> Result<(), ()> {
    let mut p = Periphals::take().ok_or(())?;
    // initialize pin 8 (led) as output
    let mut gpio_8 = p.init_gpio(0, 8).into_output().map_err(|_| ())?;
    // initialize pin 10 as input
    let gpio_10 = p.init_gpio(0, 10).into_input().map_err(|_| ())?;

    // open a uart connections over pins 2 (rx) and 3 (tx)
    let uart_pins = Pins {
        rx: p.init_gpio(0, 2).into_input().map_err(|_| ())?,
        tx: p.init_gpio(0, 3).into_output().map_err(|_| ())?,
        cts: None,
        rts: None,
    };
    let mut uart = Uart::new(uart_pins).map_err(|_| ())?;

    loop {
        // turn the led on
        gpio_8.set_high().map_err(|_| ())?;

        // read the value of pin 10
        let is_val_10_hi = gpio_10.is_high().map_err(|_| ())?;
        let msg = if is_val_10_hi {
            "val 10 hi"
        } else {
            "val 10 lo"
        };
        // write it over uart and to the console
        uart.write_str(&msg).map_err(|_| ())?;
        print!(msg);

        // delay for a second
        p.delay_ms(1_000);

        // turn the led off and wait another second
        gpio_8.set_low().map_err(|_| ())?;
        p.delay_ms(1_000);
    }
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
