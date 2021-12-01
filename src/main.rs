// Necessary, so that the `app_main` symbol exported by the `binstart` feature of esp-if-sys is linked
use esp_idf_sys;

use log::{info, LevelFilter};
use logging::SimpleLogger;
use wasmi::{ImportsBuilder, Module, ModuleInstance, StackRecycler};

mod bytes;
mod logging;
mod runtime;

use bytes::WASM_BYTES;
use runtime::{Runtime, UartModuleImportResolver};

static LOGGER: SimpleLogger = SimpleLogger;

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();

    info!("Hello, riscv!");

    let module = Module::from_buffer(&WASM_BYTES).unwrap();
    info!("Module loaded successfully!");

    // instantiate a module and pass it the import resolver
    let main = ModuleInstance::new(
        &module,
        &ImportsBuilder::new().with_resolver("env", &UartModuleImportResolver),
    );

    match main {
        Err(err) => {
            info!("Error: {}", err);
        }
        Ok(main) => {
            let main = main.assert_no_start();

            // fetch the memory of the module (needed for the write and read buffer)
            let memory_export = main.export_by_name("memory").unwrap();
            let memory = memory_export.as_memory().unwrap();

            let mut runtime = Runtime::new(memory);
            let mut stack_rec = StackRecycler::with_limits(84 * 1024, 84 * 1024);

            info!("Calling the start method!");
            let result = main.invoke_export_with_stack("start", &[], &mut runtime, &mut stack_rec);
            info!("All went well? {:#?}", result.is_ok());
        }
    }
}
