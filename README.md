# Wasm on ESP32-C3

This is a demonstration of running a WebAssembly interpreter (in this case [wasmi](https://github.com/paritytech/wasmi)) on the ESP32-C3 RISC-V Microcontroller.
This is a very basic demonstration, where the runtime only provides a few functions to call from WebAssembly. A usage of the API is shown in the languages C, C++ and Rust.
The runtime's functionality includes reading and writing to GPIOs and communication over a UART connection.

This demo is build upon Espressifs effort of porting the Rust standard library to their boards, running on the [esp-idf](https://github.com/espressif/esp-idf) development framework.
In order to run the demonstration you would need to use the latest Rust nightly compiler. Further Instructions can be found under [setup](#Setup).

## How this demo works

The compiled WASM application code will be flashed onto the board as part of a static variable in the Rust code (found in [`src/bytes.rs`](src/bytes.rs)). At runtime these bytes will be loaded and
executed. To show an example usage, this demo involves three subprojects in [C](application-c), [C++](application-cpp) and [Rust](application-rs), which all include a small abstraction
of the runtimes API and a program that basically implements the following example control flow:

```
PROGRAM example_usage:
    pin_8 = Pin 8 (LED) as output
    pin_10 = Pin 10 as input

    uart = uart over pins 2 (rx) and 3 (tx)

    LOOP:
        pin_8.set_high()

        value_10 = pin_10.is_high()
        if value_10 then
            msg = "value 10 is hi"
        else
            msg = "value 10 is lo"
        end if

        print(msg)
        uart.write(msg)

        sleep(1 second)

        pin_8.set_low()
        sleep(1 second)
    end LOOP
end PROGRAM
```

Each executable contains a `start` functions that implements the above behaviour. When loaded, the runtime needs to resolve each API call. This
is done via the `Runtime` struct defined in [`src/runtime.rs`](src/runtime.rs). The runtime object also holds information about the current 
state of the program, like opened UART connections and initialized Gpios.

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

To build each application, follow the specific instructions.
