use crate::error::ErrorCode;

/// Extern functions that define the API of our runtime. All the types, methods and
/// abstractions in this crate build upon this API.
#[link(wasm_import_module = "env")]
extern "C" {
    pub fn print(offset: *const u8, len: u32);

    pub fn uart_init(
        handle: *mut u8,
        txd_port: u32,
        txd_pin: u32,
        rxd_port: u32,
        rxd_pin: u32,
        cts_port: *const u32,
        cts_pin: *const u32,
        rts_port: *const u32,
        rts_pin: *const u32,
    ) -> ErrorCode;

    pub fn uart_write(handle: u8, word: u8) -> ErrorCode;

    pub fn uart_read(handle: u8, value: *mut u8) -> ErrorCode;

    pub fn gpio_init(port: u32, pin: u32, is_input: bool) -> ErrorCode;

    pub fn gpio_write(port: u32, pin: u32, value: u32) -> ErrorCode;

    pub fn gpio_read(port: u32, pin: u32, value: *mut u32) -> ErrorCode;

    pub fn delay_ms(ms: u32);
}
