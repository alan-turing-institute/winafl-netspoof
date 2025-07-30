mod connections;
mod drcore;
mod drwrap;
mod ffi;
mod modbus;
mod utils;
mod wrappers;

use drcore::{get_proc_address, log, utf8_name_of_module};
use ffi::{app_pc, module_data_t};
use std::os::raw::c_void;
use std::panic;
use utils::Boolean;

/// Called by DynamoRio when each C module is loaded.
/// Each .dll will be a module, and the main binary itself will be comprised of one or more
/// modules.
#[unsafe(no_mangle)]
pub extern "C" fn module_load_event(
    _drctx: *mut c_void,
    module_ptr: *const module_data_t,
    _loaded: bool,
) {
    // Override the panic hook to log the DynamoRio logger when a panic occurs.
    panic::set_hook(Box::new(|panic_info| {
        log(&format!("panic occurred: {panic_info}"));
    }));

    let module;
    unsafe {
        module = *module_ptr;
    }
    let mod_name = match utf8_name_of_module(module) {
        Ok(name) => name,
        Err(utf8_name_error) => {
            log(&format!(
                "[module load] failed to read the module name: {:?}",
                utf8_name_error
            ));
            return;
        }
    };
    log(&format!("[module load] loading {mod_name}"));

    if module.entry_point.is_null() {
        log(
            "[module load] module entry point is a null pointer, will not search for proc_addresses.",
        );
        return;
    };

    // Only hook functions in ws2_32.dll to avoid additionally hooking re-exported versions of
    // the functinos in other modules.
    if &mod_name.to_lowercase() == "ws2_32.dll" {
        let mod_base_addr = module.entry_point as app_pc;

        // wrap "connect"
        unsafe {
            if let Some(addr) = get_proc_address(mod_base_addr, "connect") {
                match ffi::drwrap_wrap(addr as *mut u8, Some(wrappers::wrap_pre_connect), None)
                    .as_bool()
                {
                    true => log(&format!("[module load] wrapped connect @ 0x{:?}", addr)),
                    false => log(&format!(
                        "[module load] failed to wrap connect @ 0x{:?}",
                        addr
                    )),
                }
            };
        };

        // wrap "send"
        unsafe {
            if let Some(addr) = get_proc_address(mod_base_addr, "send") {
                match ffi::drwrap_wrap(addr as *mut u8, Some(wrappers::wrap_pre_send), None)
                    .as_bool()
                {
                    true => log(&format!("[module load] wrapped send @ 0x{:?}", addr)),
                    false => log(&format!("[module load] failed to wrap send @ 0x{:?}", addr)),
                }
            };
        };

        // wrap "recv"
        unsafe {
            if let Some(addr) = get_proc_address(mod_base_addr, "recv") {
                match ffi::drwrap_wrap(addr as *mut u8, Some(wrappers::wrap_pre_recv), None)
                    .as_bool()
                {
                    true => log(&format!("[module load] wrapped recv @ 0x{:?}", addr)),
                    false => log(&format!("[module load] failed to wrap recv @ 0x{:?}", addr)),
                }
            };
        };
    };
    return;
}
