use esp_idf_hal::gpio::Gpio1;
use esp_idf_hal::gpio::Gpio3;
use esp_idf_hal::gpio::Unknown;
use esp_idf_hal::serial::Serial;
use std::fmt::Write;
use wasmi::Module;
// Necessary, so that the `app_main` symbol exported by the `binstart` feature of esp-if-sys is linked
use esp_idf_sys;

use wasmi::{
    Externals, FuncInstance, ImportsBuilder, ModuleImportResolver, ModuleInstance, RuntimeValue,
    Signature, StackRecycler, TrapKind, ValueType,
};

use esp_idf_hal::prelude::*;
use esp_idf_hal::serial;

// A wasm module that calls the uart write, read and print method.
#[link_section = ".iram.data"]
static WASM_BYTES: [u8; 554] = [
    0, 97, 115, 109, 1, 0, 0, 0, 1, 27, 5, 96, 2, 127, 127, 1, 127, 96, 2, 127, 127, 0, 96, 1, 127,
    1, 127, 96, 0, 0, 96, 3, 127, 127, 127, 1, 127, 2, 38, 3, 3, 101, 110, 118, 5, 119, 114, 105,
    116, 101, 0, 0, 3, 101, 110, 118, 7, 112, 114, 105, 110, 116, 108, 110, 0, 1, 3, 101, 110, 118,
    4, 114, 101, 97, 100, 0, 2, 3, 3, 2, 3, 4, 5, 3, 1, 0, 1, 6, 22, 3, 127, 1, 65, 128, 244, 3,
    11, 127, 0, 65, 224, 244, 3, 11, 127, 0, 65, 224, 244, 3, 11, 7, 45, 4, 6, 109, 101, 109, 111,
    114, 121, 2, 0, 5, 115, 116, 97, 114, 116, 0, 3, 10, 95, 95, 100, 97, 116, 97, 95, 101, 110,
    100, 3, 1, 11, 95, 95, 104, 101, 97, 112, 95, 98, 97, 115, 101, 3, 2, 10, 159, 2, 2, 99, 1, 1,
    127, 35, 0, 65, 128, 2, 107, 34, 0, 36, 0, 32, 0, 65, 1, 106, 65, 0, 65, 255, 1, 16, 4, 26, 2,
    64, 2, 64, 65, 128, 244, 3, 65, 12, 16, 0, 13, 0, 65, 165, 244, 3, 65, 19, 16, 1, 2, 64, 32, 0,
    65, 1, 106, 16, 2, 13, 0, 65, 208, 244, 3, 65, 16, 16, 1, 12, 2, 11, 65, 184, 244, 3, 65, 24,
    16, 1, 12, 1, 11, 65, 140, 244, 3, 65, 25, 16, 1, 11, 32, 0, 65, 128, 2, 106, 36, 0, 11, 184,
    1, 1, 3, 127, 2, 64, 32, 2, 69, 13, 0, 32, 2, 65, 7, 113, 33, 3, 65, 0, 33, 4, 2, 64, 32, 2,
    65, 127, 106, 65, 7, 73, 13, 0, 32, 2, 65, 120, 113, 33, 5, 65, 0, 33, 4, 3, 64, 32, 0, 32, 4,
    106, 34, 2, 32, 1, 58, 0, 0, 32, 2, 65, 7, 106, 32, 1, 58, 0, 0, 32, 2, 65, 6, 106, 32, 1, 58,
    0, 0, 32, 2, 65, 5, 106, 32, 1, 58, 0, 0, 32, 2, 65, 4, 106, 32, 1, 58, 0, 0, 32, 2, 65, 3,
    106, 32, 1, 58, 0, 0, 32, 2, 65, 2, 106, 32, 1, 58, 0, 0, 32, 2, 65, 1, 106, 32, 1, 58, 0, 0,
    32, 5, 32, 4, 65, 8, 106, 34, 4, 71, 13, 0, 11, 11, 32, 3, 69, 13, 0, 32, 0, 32, 4, 106, 33, 2,
    3, 64, 32, 2, 32, 1, 58, 0, 0, 32, 2, 65, 1, 106, 33, 2, 32, 3, 65, 127, 106, 34, 3, 13, 0, 11,
    11, 32, 0, 11, 11, 104, 1, 0, 65, 128, 244, 3, 11, 96, 72, 101, 108, 108, 111, 32, 87, 111,
    114, 108, 100, 33, 85, 102, 102, 44, 32, 112, 97, 110, 105, 99, 32, 100, 117, 114, 105, 110,
    103, 32, 119, 114, 105, 116, 101, 33, 33, 87, 114, 105, 116, 116, 101, 110, 32, 115, 117, 99,
    99, 101, 115, 102, 117, 108, 108, 121, 85, 102, 102, 44, 32, 112, 97, 110, 105, 99, 32, 100,
    117, 114, 105, 110, 103, 32, 114, 101, 97, 100, 33, 33, 82, 101, 97, 100, 32, 115, 117, 99, 99,
    101, 115, 102, 117, 108, 108, 121,
];

/// Runtime that handles call to the host machine. Holds information about the current UART connection
/// that is exposed to the WASM module and the memory region the WASM module operates in.
struct Runtime<'a> {
    uart: Serial<serial::UART1, Gpio1<Unknown>, Gpio3<Unknown>>,
    memory: &'a mut [u8],
}

impl<'a> Runtime<'a> {
    /// Creates an instance with a specific buffer that represents the runtimes memory.
    fn new(memory: &'a mut [u8]) -> Self {
        Self {
            memory,
            uart: Self::setup_uart(),
        }
    }

    /// Write via UART. Writes the content of a buffer to the UART interface.
    fn write(&mut self, offset: usize, len: usize) -> u32 {
        // extract the buffer from memory and write It
        let buf = &self.memory[offset..offset + len];
        let message_as_string = unsafe { core::str::from_utf8_unchecked(buf) };
        println!("Trying to write: {}", message_as_string);

        write!(self.uart, "{:}", message_as_string).map_or(1, |_| 0)
    }

    /// Read a single byte via UART the uart interface.
    fn read_byte(&mut self, offset: usize) -> u32 {
        let buf = &mut self.memory[offset..offset + 1];

        match self.uart.read() {
            Ok(word) => buf[0] = word,
            Err(e) => {
                eprintln!("Error in read: {:#?}", e);
                return 1;
            }
        };

        0
    }

    fn println(&mut self, offset: usize, len: usize) {
        let bytes = &self.memory[offset..offset + len];

        let msg = unsafe { core::str::from_utf8_unchecked(bytes) };
        println!("{}", msg);
    }

    fn setup_uart() -> Serial<serial::UART1, Gpio1<Unknown>, Gpio3<Unknown>> {
        let peripherals = Peripherals::take().unwrap();
        let pins = peripherals.pins;

        let config = serial::config::Config::default().baudrate(Hertz(115_200));

        serial::Serial::new(
            peripherals.uart1,
            serial::Pins {
                tx: pins.gpio1,
                rx: pins.gpio3,
                cts: None,
                rts: None,
            },
            config,
        )
        .unwrap()
    }
}

/// Internal index of the functions.
const WRITE_INDEX: usize = 0;
const READ_INDEX: usize = 1;
const PRINT_INDEX: usize = 2;

impl<'a> Externals for Runtime<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: wasmi::RuntimeArgs,
    ) -> Result<Option<wasmi::RuntimeValue>, wasmi::Trap> {
        let offset: i32 = args.nth(0);

        // find the right function and execute it
        match index {
            WRITE_INDEX => {
                println!("Write called!");

                let len: i32 = args.nth(1);
                let result = self.write(offset as usize, len as usize);

                Ok(Some(RuntimeValue::I32(result as i32)))
            }
            READ_INDEX => {
                println!("Read called!");

                let result = self.read_byte(offset as usize);

                Ok(Some(RuntimeValue::I32(result as i32)))
            }
            PRINT_INDEX => {
                let len: i32 = args.nth(1);
                self.println(offset as usize, len as usize);
                Ok(None)
            }
            _ => Err(wasmi::Trap::new(TrapKind::UnexpectedSignature)),
        }
    }
}

/// Resolves external functions on the host system.
struct UartModuleImportResolver;

impl<'a> ModuleImportResolver for UartModuleImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &wasmi::Signature,
    ) -> Result<wasmi::FuncRef, wasmi::Error> {
        match field_name {
            "write" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                WRITE_INDEX,
            )),
            "read" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                READ_INDEX,
            )),
            "println" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                PRINT_INDEX,
            )),
            x => Err(wasmi::Error::Function(format!("unknown function {}", x))),
        }
    }
}

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    println!("Hello, riscv!");

    let module = Module::from_buffer(&WASM_BYTES).unwrap();
    println!("Module loaded successfully!");

    // instantiate a module and pass it the import resolver
    let main = ModuleInstance::new(
        &module,
        &ImportsBuilder::new().with_resolver("env", &UartModuleImportResolver),
    );

    match main {
        Err(err) => {
            println!("Error while instantiationg module: {}", err);
        }
        Ok(main) => {
            let main = main.assert_no_start();

            // fetch the memory of the module (needed for the write and read buffer)
            let memory_export = main.export_by_name("memory").unwrap();
            let memory = memory_export.as_memory().unwrap();

            // extract the concrete memory and pass it to the runtime object
            let mut bytes = memory
                .get(0, wasmi::memory_units::Bytes::from(memory.current_size()).0)
                .unwrap();
            let mut runtime = Runtime::new(&mut bytes);
            let mut stack_rec = StackRecycler::with_limits(84 * 1024, 84 * 1024);

            let result = main.invoke_export_with_stack("start", &[], &mut runtime, &mut stack_rec);
            println!("All went well? {:#?}", result.is_ok());
        }
    }
}
