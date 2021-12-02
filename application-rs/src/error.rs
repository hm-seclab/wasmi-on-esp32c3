pub(crate) type ErrorCode = i32;

/// An error in the Wasm context.
#[derive(Debug)]
pub enum WasmError {
    /// An error returned by the runtime,
    /// identified by an error code.
    RuntimeError(ErrorCode),
}
