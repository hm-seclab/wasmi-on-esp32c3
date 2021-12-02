# Rust Example

As the other two examples in this Repo, this application implements a small abstraction of the runtimes API.
The example program mentioned in the main README can be found in the [examples](examples) folder.

This abstraction follows the API defined by [embedded_hal](https://github.com/rust-embedded/embedded-hal). It
by far doesn't cover every part of the API, but delivers a conform implementation for Gpios, serial connections and timers.

## Building this example

This code gets compiled to the rust target `wasm32-unknown-unknown`. As the Rust compiler includes a huge stack in WASM, (see [here](https://github.com/rust-lang/rust/blob/a16f686e4a0ea15dcd3b5aa3db7b1cba27bb9453/compiler/rustc_target/src/spec/wasm_base.rs#L13-L17)), this code is compiled with the option `-z stack-size=32768` (see [config.toml](.cargo/config.toml)).
The library is seperated from the example, as they live in seperate folders ([src](src) and [examples](examples)). The library tries to avoid any
formatting code as this bloats the binary size in Rust. It is furthermore recommended to compile the program in release mode with the following profile:
```toml
[profile.release]
panic           = "abort" # aborts on panic, disabling most error formatting
codegen-units   = 1
opt-level       = 'z' # optimized for the size
```
The [example](examples/pins_and_gpio.rs) avoids the formatting bloat by mapping all errors to the unit type `()` and returning in
an error case. This can be more difficult to debug, but makes sure that the resulting binary is small.

To compile this example, execute the following command in this folder:

```bash
cargo b --release --example pins_and_gpio
```
