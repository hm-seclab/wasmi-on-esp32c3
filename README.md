# Wasm on ESP32-C3

This is a demonstration of running a WebAssembly interpreter (in this case [wasmi](https://github.com/paritytech/wasmi)) on the ESP32-C3 RISC-V Microcontroller.
This is a very basic demonstration, where the runtime only provides three functions to call from WebAssembly: `write`, `read` and `println`, where `write` and `read` perform the
corresponding UART operation over the pins 1 (TX) and 3 (RX) (without CTS and RTS).

This demo is build upon espressifs effort of porting the Rust standard library to their boards, running on the [esp-idf](https://github.com/espressif/esp-idf) development framework.
In order to run the demonstration you would need to use the latest Rust nightly compiler. Further Instructions can be found under [setup](#Setup).

## How this demo works

The compiled WASM application code will be flashed onto the board as part of a static variable in the Rust code. The bytes are the output of compiling the following Rust program to WebAssembly:

```rust
// we don't need the standard library here
#![no_std]

// define the extern functions provided by the runtime, this will result
// in WebAssembly `imports`.
extern "C" {
    // `write` takes a buffer and it's length and then writes the contents
    // of the buffer on the UART interface.
    pub fn write(offset: *const u8, len: u32) -> u32;
    // `read` reads a single byte from the UART interface. 
    pub fn read(offset: *const u8) -> u32;
    // prints to stdout.
    pub fn println(offset: *const u8, len: u32);
}

// for convenience: a macro for printing strings (no formatting available).
macro_rules! println {
    ($arg:tt) => {
        unsafe { println($arg.as_ptr(), $arg.len() as u32) }
    };
}

// The main method that is exported by the WASM module.
#[no_mangle]
pub fn start() {
    let buffer = [0_u8; 255];

    let message = "Hello World!";
    // write over UART and check the result 
    let return_val = unsafe { write(message.as_ptr(), message.len() as u32) };
    if return_val != 0 {
        println!("Uff, panic during write!!");
        return;
    }
    println!("Written succesfully");

    // read over uart and check the result
    let return_val = unsafe { read(buffer.as_ptr()) };
    if return_val != 0 {
        println!("Uff, panic during read!!");
        return;
    }
    println!("Read succesfully");
}

// our custom panic handler as #![no_std] disallows us to use the standard one.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

This code gets compiled to the rust target `wasm32-unknown-unknown`. The `wasmi` runtime then loads the module.
In order for the above code to work, the runtime needs to resolve the imported functions `read`, `write` and `println`, which
are defined in the file `src/main.rs` and use the underlying Esp-HAL for instantiating a UART connection.

After the module is loaded, the runtime executes the `start` function, that's exported by the WASM program above.

## Setup

If you don't have rustup installed yet, follow the instructions on the [rustup.rs](rustup.rs) site.
Then install the nightly Rust compiler toolchain via:

```
rustup install nightly
rustup component add rust-src --toolchain nightly
```
To build this project you also need a recent Clang compiler, the [Clang getting started](https://clang.llvm.org/get_started.html) website explains the available install options.

It is also necessary to install the following cargo subcommands...
```bash
cargo install ldproxy # linker proxy, used by this project
cargo install espflash # for flashing the binary onto the board
cargo install espmonitor # for monitoring the system state and accessing stdout
```


... and the platform.io core package. Installation instructions can be found [here](https://docs.platformio.org/en/latest/core/installation.html).

After a succsefull installation of the tools you are ready to build the project:
```
cargo build --release
```

Then flash the binary with the following command:
```
espflash /dev/ttyUSB0 target/riscv32imc-esp-espidf/release/wasm-on-esp32c3
```

- Replace `/dev/ttyUSB0` with the USB port where you've connected the board

Then monitor the app via:
```
espmonitor /dev/ttyUSB0
```
