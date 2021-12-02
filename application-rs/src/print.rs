/// A struct that handles printing.
pub struct WasmPrint;

impl WasmPrint {
    /// Write a string to the console.
    pub fn write_str(&mut self, s: &str) {
        unsafe { crate::runtime::print(s.as_ptr(), s.len() as u32) };
    }
}

#[macro_export]
macro_rules! print {
    ($arg:tt) => {{
        let mut p = crate::print::WasmPrint;
        p.write_str($arg)
    }};
}
