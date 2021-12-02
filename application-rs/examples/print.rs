#![no_main]
use wasm_embedded_hal::print;

#[no_mangle]
fn start() {
    print!("hello, world");
}
