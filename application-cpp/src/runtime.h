#pragma once
#define WASM_IMPORT(function, signature) \
    __attribute__((import_module("env"), import_name(function))) signature

WASM_IMPORT("gpio_read", int gpio_read(unsigned int port, unsigned int pin,
                                       unsigned int* ptr));
WASM_IMPORT("gpio_write", int gpio_write(unsigned int port, unsigned int pin,
                                         unsigned int value));
WASM_IMPORT("gpio_deinit",
            int gpio_deinit(unsigned int port, unsigned int pin));
WASM_IMPORT("gpio_init",
            int gpio_init(unsigned int port, unsigned int pin, int is_input));
WASM_IMPORT("delay_ms", void delay_ms(unsigned int ms));
WASM_IMPORT("print", void print(char const* offset, int len));
WASM_IMPORT("uart_init",
            int uart_init(unsigned char* handle, unsigned int tx_port,
                          unsigned int tx_pin, unsigned int rx_port,
                          unsigned int rx_pin, unsigned int* cts_port,
                          unsigned int* cts_pin, unsigned int* rts_port,
                          unsigned int* rts_pin));
WASM_IMPORT("uart_write",
            int uart_write(unsigned int handle, unsigned char word));
WASM_IMPORT("uart_read",
            int uart_read(unsigned int handle, unsigned char* word));
