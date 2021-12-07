use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::serial;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::Gpio1;
use esp_idf_hal::gpio::Gpio10;
use esp_idf_hal::gpio::Gpio11;
use esp_idf_hal::gpio::Gpio18;
use esp_idf_hal::gpio::Gpio19;
use esp_idf_hal::gpio::Gpio2;
use esp_idf_hal::gpio::Gpio20;
use esp_idf_hal::gpio::Gpio21;
use esp_idf_hal::gpio::Gpio3;
use esp_idf_hal::gpio::Gpio4;
use esp_idf_hal::gpio::Gpio5;
use esp_idf_hal::gpio::Gpio6;
use esp_idf_hal::gpio::Gpio7;
use esp_idf_hal::gpio::Gpio8;
use esp_idf_hal::gpio::Gpio9;
use esp_idf_hal::gpio::GpioPin;
use esp_idf_hal::gpio::Input;
use esp_idf_hal::gpio::Output;
use esp_idf_hal::gpio::Unknown;
use esp_idf_hal::serial::config::Config;
use esp_idf_hal::serial::Pins;
use esp_idf_hal::serial::Serial;
use esp_idf_hal::serial::UART1;
use esp_idf_sys::EspError;
use log::info;
use std::collections::HashMap;
use wasmi::MemoryRef;
use wasmi::{
    Externals, FuncInstance, ModuleImportResolver, RuntimeValue, Signature, TrapKind, ValueType,
};

use esp_idf_hal::prelude::*;

/// A macro for configuring a certain pin (identified by it's type)
/// as Input or Output. E.g. `configure_pin!(esp_idf_hal::gpio::Gpio1, Input)`.
macro_rules! configure_pin {
    ($pin_ty:ty, Input) => {{
        let gpio = unsafe { <$pin_ty>::new() };
        let gpio = gpio.into_input().unwrap();
        gpio
    }};

    ($pin_ty:ty, Output) => {{
        let gpio = unsafe { <$pin_ty>::new() };
        let gpio = gpio.into_output().unwrap();
        gpio
    }};
}

/// A macro for configure a pin as input or output and insert it into
/// either a map for input pins or one for output pins. Used by the
/// runtime to reduce duplicated code.
macro_rules! configure_and_insert_pin {
    ($pin_ty:ty, $is_inp:expr, $key:expr ,$map_in:expr, $map_out:expr) => {{
        if $is_inp {
            let pin = configure_pin!($pin_ty, Input);
            $map_in.insert($key, Box::new(pin));
        } else {
            let pin = configure_pin!($pin_ty, Output);
            $map_out.insert($key, Box::new(pin));
        }
    }};
}

/// A pin, defined by it's port and pin number.
type RuntimePin = (u32, u32);
/// The type for the handles that are given out for a UART connection.
type UartHandle = u8;
// The error code that gets returned to the user.
type ErrorCode = i32;

/// Convenience trait for creating trait objects, statisfies both serial read and write.
trait ReadAndWrite: serial::Read<u8, Error = EspError> + serial::Write<u8, Error = EspError> {}

/// Tell the compiler that the trait is implemented for esp_idf_hals Serial connection type.
impl<
        TX: esp_idf_hal::gpio::OutputPin,
        RX: esp_idf_hal::gpio::InputPin,
        CTS: esp_idf_hal::gpio::InputPin,
        RTS: esp_idf_hal::gpio::OutputPin,
    > ReadAndWrite for Serial<UART1, TX, RX, CTS, RTS>
{
}

/// Runtime that handles call to the host machine. Holds information about the current UART connection
/// that is exposed to the WASM module, the memory region the WASM module operates in and the Gpio pins
/// that are being used.
pub(crate) struct Runtime<'a> {
    memory: &'a MemoryRef,
    handle_count: u8,
    uart_connections: HashMap<UartHandle, Box<dyn ReadAndWrite>>,
    gpio_input_mapping: HashMap<RuntimePin, Box<dyn InputPin<Error = EspError>>>,
    gpio_output_mapping: HashMap<RuntimePin, Box<dyn OutputPin<Error = EspError>>>,
}

impl<'a> Runtime<'a> {
    /// Creates an instance with a reference to the instances memory.
    pub(crate) fn new(memory: &'a MemoryRef) -> Self {
        Self {
            memory,
            handle_count: 1,
            uart_connections: Default::default(),
            gpio_input_mapping: HashMap::new(),
            gpio_output_mapping: HashMap::new(),
        }
    }

    /// Inititalize a new uart connection over the given pins.
    /// Currently it is only possible to open one UART connection
    /// per runtime, but this will change soon.
    fn uart_init(
        &mut self,
        tx: RuntimePin,
        rx: RuntimePin,
        cts: Option<RuntimePin>,
        rts: Option<RuntimePin>,
        handle: u32,
    ) -> ErrorCode {
        // for the moment: allow only one uart connection per runtime
        if self.uart_connections.len() > 0 {
            return -1;
        }
        let peripherals = Peripherals::take().unwrap();
        // Setup the pins
        let tx_pin = match Self::get_output_pin_by_nr(tx.1) {
            Ok(p) => p,
            Err(err) => return err.code(),
        };

        let rx_pin = match Self::get_input_pin_by_nr(rx.1) {
            Ok(p) => p,
            Err(err) => return err.code(),
        };

        let cts_pin = cts
            .map(|(_, pin_ptr)| self.memory.get_value(pin_ptr).unwrap_or(0))
            .map(|pin| Self::get_input_pin_by_nr(pin).ok())
            .unwrap_or_default();

        let rts_pin = rts
            .map(|(_, pin_ptr)| self.memory.get_value(pin_ptr).unwrap_or(0))
            .map(|pin| Self::get_output_pin_by_nr(pin).ok())
            .unwrap_or_default();

        let pins = Pins {
            tx: tx_pin,
            rx: rx_pin,
            cts: cts_pin,
            rts: rts_pin,
        };

        // create a config
        let config = Config::default().baudrate(Hertz(115_200));

        // initialize a serial connection over the defined pins
        let serial = match Serial::new(peripherals.uart1, pins, config) {
            Ok(ser) => ser,
            Err(err) => return err.code(),
        };

        // save the handle so that the WASM code can acess it
        let res = self
            .memory
            .set_value(handle, self.handle_count)
            .map_or(1, |_| 0);

        // save the connection as a trait object
        self.uart_connections
            .insert(self.handle_count, Box::new(serial));

        self.handle_count += 1;

        res
    }

    /// Initialize a pin as input pin and return it as a generic `GpioPin`.
    fn get_input_pin_by_nr(nr: u32) -> Result<GpioPin<Input>, EspError> {
        match nr {
            1 => Ok(unsafe { Gpio1::<Unknown>::new() }.into_input()?.degrade()),
            2 => Ok(unsafe { Gpio2::<Unknown>::new() }.into_input()?.degrade()),
            3 => Ok(unsafe { Gpio3::<Unknown>::new() }.into_input()?.degrade()),
            4 => Ok(unsafe { Gpio4::<Unknown>::new() }.into_input()?.degrade()),
            5 => Ok(unsafe { Gpio5::<Unknown>::new() }.into_input()?.degrade()),
            6 => Ok(unsafe { Gpio6::<Unknown>::new() }.into_input()?.degrade()),
            7 => Ok(unsafe { Gpio7::<Unknown>::new() }.into_input()?.degrade()),
            8 => Ok(unsafe { Gpio8::<Unknown>::new() }.into_input()?.degrade()),
            9 => Ok(unsafe { Gpio9::<Unknown>::new() }.into_input()?.degrade()),
            10 => Ok(unsafe { Gpio10::<Unknown>::new() }.into_input()?.degrade()),
            11 => Ok(unsafe { Gpio11::<Unknown>::new() }.into_input()?.degrade()),
            18 => Ok(unsafe { Gpio18::<Unknown>::new() }.into_input()?.degrade()),
            19 => Ok(unsafe { Gpio19::<Unknown>::new() }.into_input()?.degrade()),
            20 => Ok(unsafe { Gpio20::<Unknown>::new() }.into_input()?.degrade()),
            21 => Ok(unsafe { Gpio21::<Unknown>::new() }.into_input()?.degrade()),
            _ => Err(EspError::from(2).unwrap()),
        }
    }

    /// Initialize a pin as output pin and return it as a generic `GpioPin`.
    fn get_output_pin_by_nr(nr: u32) -> Result<GpioPin<Output>, EspError> {
        match nr {
            1 => Ok(unsafe { Gpio1::<Unknown>::new() }.into_output()?.degrade()),
            2 => Ok(unsafe { Gpio2::<Unknown>::new() }.into_output()?.degrade()),
            3 => Ok(unsafe { Gpio3::<Unknown>::new() }.into_output()?.degrade()),
            4 => Ok(unsafe { Gpio4::<Unknown>::new() }.into_output()?.degrade()),
            5 => Ok(unsafe { Gpio5::<Unknown>::new() }.into_output()?.degrade()),
            6 => Ok(unsafe { Gpio6::<Unknown>::new() }.into_output()?.degrade()),
            7 => Ok(unsafe { Gpio7::<Unknown>::new() }.into_output()?.degrade()),
            8 => Ok(unsafe { Gpio8::<Unknown>::new() }.into_output()?.degrade()),
            9 => Ok(unsafe { Gpio9::<Unknown>::new() }.into_output()?.degrade()),
            10 => Ok(unsafe { Gpio10::<Unknown>::new() }.into_output()?.degrade()),
            11 => Ok(unsafe { Gpio11::<Unknown>::new() }.into_output()?.degrade()),
            18 => Ok(unsafe { Gpio18::<Unknown>::new() }.into_output()?.degrade()),
            19 => Ok(unsafe { Gpio19::<Unknown>::new() }.into_output()?.degrade()),
            20 => Ok(unsafe { Gpio20::<Unknown>::new() }.into_output()?.degrade()),
            21 => Ok(unsafe { Gpio21::<Unknown>::new() }.into_output()?.degrade()),
            _ => Err(EspError::from(2).unwrap()),
        }
    }

    /// Write via UART. Writes a single byte over the uart interface.
    /// Operates on the already hand out uart handles and calls the `write`
    /// method on the trait object.
    fn uart_write(&mut self, handle: UartHandle, word: u8) -> ErrorCode {
        match self.uart_connections.get_mut(&handle) {
            Some(connection) => connection.write(word).map_or(-1, |_| 0),
            None => 1,
        }
    }

    /// Reads a single byte via UART. Operates on the already hand out uart
    /// handles and calls the `read` method on the trait object.
    fn uart_read(&mut self, handle: UartHandle, offset: u32) -> ErrorCode {
        match self.uart_connections.get_mut(&handle) {
            Some(connection) => match connection.read() {
                Ok(word) => self.memory.set_value(offset, word).map_or(1, |_| 0),
                Err(_) => 1,
            },
            None => 1,
        }
    }

    /// Prints to the command line, helpful for debugging the WASM applications.
    fn print(&mut self, offset: u32, len: usize) {
        let bytes = self.memory.get(offset, len).unwrap();

        let msg = unsafe { core::str::from_utf8_unchecked(&bytes) };
        println!("{}", msg);
    }

    /// Initialize a gpio pin as Input or output and safe it for later.
    fn init_gpio(&mut self, port: u32, pin: u32, is_input: bool) -> ErrorCode {
        if port != 0 {
            return -1;
        }
        // reset the pin due to
        // https://github.com/esp-rs/esp-idf-hal/issues/9
        unsafe {
            esp_idf_sys::gpio_reset_pin(pin as i32);
        }

        // initialize the pin and safe it into the input or output map
        match pin {
            1 => configure_and_insert_pin!(
                Gpio1<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            2 => configure_and_insert_pin!(
                Gpio2<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            3 => configure_and_insert_pin!(
                Gpio3<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            4 => configure_and_insert_pin!(
                Gpio4<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            5 => configure_and_insert_pin!(
                Gpio5<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            6 => configure_and_insert_pin!(
                Gpio6<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            7 => configure_and_insert_pin!(
                Gpio7<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            8 => configure_and_insert_pin!(
                Gpio8<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            9 => configure_and_insert_pin!(
                Gpio9<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            10 => configure_and_insert_pin!(
                Gpio10<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            11 => configure_and_insert_pin!(
                Gpio11<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            18 => configure_and_insert_pin!(
                Gpio18<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            19 => configure_and_insert_pin!(
                Gpio19<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            20 => configure_and_insert_pin!(
                Gpio20<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            21 => configure_and_insert_pin!(
                Gpio21<Unknown>,
                is_input,
                (port, pin),
                self.gpio_input_mapping,
                self.gpio_output_mapping
            ),
            _ => return -1,
        }

        info!(
            "Initialized pin {} as {}",
            pin,
            if is_input { "Input" } else { "Output" }
        );

        0
    }

    /// Deinitializes a Gpio and frees the underlying ressources.
    fn deinit_gpio(&mut self, port: u32, pin: u32) -> ErrorCode {
        if let Some((_, gpio)) = self.gpio_output_mapping.remove_entry(&(port, pin)) {
            drop(gpio);
        }
        0
    }

    /// Read the current status from a gpio pin and safe it to the memory location
    /// specified by the WASM code.
    fn read_gpio(&mut self, port: u32, pin: u32, offset: u32) -> ErrorCode {
        match self.gpio_input_mapping.get(&(port, pin)) {
            Some(gpio) => {
                info!("reading from pin {}", pin);
                match gpio.is_high() {
                    Ok(value) => self.memory.set_value(offset, value as u8).map_or(1, |_| 0),
                    Err(_) => -1,
                }
            }
            None => -1,
        }
    }

    /// Write a value to a gpio pin.
    fn write_gpio(&mut self, port: u32, pin: u32, value: u32) -> ErrorCode {
        match self.gpio_output_mapping.get_mut(&(port, pin)) {
            Some(gpio) => {
                info!("Setting gpio {} to {}", pin, value);
                if value == 0 {
                    gpio.set_low()
                } else {
                    gpio.set_high()
                }
                .map_or(-1, |_| 0)
            }
            None => -1,
        }
    }

    /// Delay the execution.
    fn delay_ms(&self, ms: u32) {
        let mut ets = Ets;
        info!("Delaying for {} ms", ms);
        ets.delay_ms(ms);
    }
}

/// Internal index of the functions.
const UART_WRITE_INDEX: usize = 0;
const UART_READ_INDEX: usize = 1;
const UART_INIT_INDEX: usize = 2;
const PRINT_INDEX: usize = 3;
const GPIO_WRITE_INDEX: usize = 4;
const GPIO_READ_INDEX: usize = 5;
const GPIO_INIT_INDEX: usize = 6;
const GPIO_DEINIT_INDEX: usize = 7;
const DELAY_MS_INDEX: usize = 8;

/// Needed for resolving the functions and call them from WASM.
impl<'a> Externals for Runtime<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: wasmi::RuntimeArgs,
    ) -> Result<Option<wasmi::RuntimeValue>, wasmi::Trap> {
        // find the right function and execute it
        // always make sure to fetch the right arguments
        match index {
            UART_WRITE_INDEX => {
                info!("UART Write called!");
                let handle: u8 = args.nth(0);
                let word: u8 = args.nth(1);
                let result = self.uart_write(handle, word);

                Ok(Some(RuntimeValue::I32(result as i32)))
            }
            UART_READ_INDEX => {
                let handle: u8 = args.nth(0);
                let ptr: u32 = args.nth(1);
                info!("UART Read called!");
                let result = self.uart_read(handle, ptr);

                Ok(Some(RuntimeValue::I32(result as i32)))
            }
            UART_INIT_INDEX => {
                let handle: u32 = args.nth(0);
                let tx_port: u32 = args.nth(1);
                let tx_pin: u32 = args.nth(2);
                let rx_port: u32 = args.nth(3);
                let rx_pin: u32 = args.nth(4);
                info!("Initializing uart");

                let result =
                    self.uart_init((tx_port, tx_pin), (rx_port, rx_pin), None, None, handle);

                Ok(Some(RuntimeValue::I32(result as i32)))
            }
            PRINT_INDEX => {
                let offset: u32 = args.nth(0);
                let len: i32 = args.nth(1);
                self.print(offset, len as usize);
                Ok(None)
            }
            GPIO_READ_INDEX => {
                let port: u32 = args.nth(0);
                let pin: u32 = args.nth(1);
                let ptr: u32 = args.nth(2);

                let res = self.read_gpio(port, pin, ptr);

                Ok(Some(RuntimeValue::I32(res)))
            }
            GPIO_WRITE_INDEX => {
                let port: u32 = args.nth(0);
                let pin: u32 = args.nth(1);
                let value: u32 = args.nth(2);
                let res = self.write_gpio(port, pin, value);

                Ok(Some(RuntimeValue::I32(res)))
            }
            DELAY_MS_INDEX => {
                let ms: u32 = args.nth(0);

                self.delay_ms(ms);

                Ok(None)
            }
            GPIO_INIT_INDEX => {
                let port: u32 = args.nth(0);
                let pin: u32 = args.nth(1);
                let input: i32 = args.nth(2);

                let res = self.init_gpio(port, pin, input == 1);

                Ok(Some(RuntimeValue::I32(res)))
            }
            GPIO_DEINIT_INDEX => {
                let port: u32 = args.nth(0);
                let pin: u32 = args.nth(1);

                let res = self.deinit_gpio(port, pin);

                Ok(Some(RuntimeValue::I32(res)))
            }
            _ => Err(wasmi::Trap::new(TrapKind::UnexpectedSignature)),
        }
    }
}

/// Resolves external functions on the host system.
pub(crate) struct UartModuleImportResolver;

impl<'a> ModuleImportResolver for UartModuleImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &wasmi::Signature,
    ) -> Result<wasmi::FuncRef, wasmi::Error> {
        match field_name {
            "uart_init" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[
                        ValueType::I32, // handle
                        ValueType::I32, // tx port
                        ValueType::I32, // tx pin
                        ValueType::I32, // rx port
                        ValueType::I32, // rx pin
                        ValueType::I32, // cts port ptr
                        ValueType::I32, // cts pin ptr
                        ValueType::I32, // rts port ptr
                        ValueType::I32, // rts pin ptr
                    ][..],
                    Some(ValueType::I32),
                ),
                UART_INIT_INDEX,
            )),
            "uart_write" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                UART_WRITE_INDEX,
            )),
            "uart_read" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                UART_READ_INDEX,
            )),
            "print" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                PRINT_INDEX,
            )),
            "gpio_read" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    Some(ValueType::I32),
                ),
                GPIO_READ_INDEX,
            )),
            "gpio_write" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    Some(ValueType::I32),
                ),
                GPIO_WRITE_INDEX,
            )),
            "gpio_init" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    Some(ValueType::I32),
                ),
                GPIO_INIT_INDEX,
            )),
            "gpio_deinit" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                GPIO_DEINIT_INDEX,
            )),
            "delay_ms" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                DELAY_MS_INDEX,
            )),
            x => Err(wasmi::Error::Function(format!("unknown function {}", x))),
        }
    }
}
