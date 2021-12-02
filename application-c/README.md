# C Example
This example application provides a small abstraction over the runtimes API (files [gpio.h](src/gpio.h) and [uart.h](src/uart.h)).
Furhtermore, the program in [main.c](src/main.cpp) implements the demo mentioned in the main README.

## Setup and building

For building this demo, a recent `clang` compiler is needed (with support for the wasm32 target), as well as the [`wasm-ld`](https://lld.llvm.org/WebAssembly.html) linker.

After a succesfull installation, the [Makefile](Makefile) handles building by the target `make all`, yielding an `out.wasm` file.
