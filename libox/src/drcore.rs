use crate::ffi::{self, _module_handle_t, app_pc, generic_func_t, module_data_t};
use crate::utils::{Boolean, ReadError, Utf8NameError, WriteError};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

impl Boolean for ffi::bool_ {
    fn as_bool(self: Self) -> bool {
        self != 0
    }
}

/// Log to the dynamorio stderr (via dr_fprintf). Adding a newline at the end of `s`.
pub fn log(s: &str) {
    let mut to_log = s.to_owned();
    to_log.push_str("\n");
    unsafe {
        let stderr = ffi::dr_get_stderr_file();
        ffi::dr_fprintf(stderr, CString::new(to_log).unwrap().as_ptr());
    }
}

/// Safely read `size` bytes from the address in `ptr`.
pub fn safe_read(ptr: *mut c_void, size: usize) -> Result<Vec<u8>, ReadError> {
    let mut buf = vec![0u8; size];
    let mut bytes_read: usize = 0;
    unsafe {
        let success =
            ffi::dr_safe_read(ptr, size, buf.as_mut_ptr() as *mut c_void, &mut bytes_read)
                .as_bool();
        if success {
            Ok(buf)
        } else {
            Err(ReadError {
                n_bytes_tried: size,
                n_bytes_read: bytes_read,
                buf,
            })
        }
    }
}

/// Safely read `size` bytes from the address in `ptr`.
pub fn safe_write(target_ptr: *mut c_void, mut to_write: Vec<u8>) -> Result<(), WriteError> {
    let mut bytes_written: usize = 0;
    unsafe {
        let success = ffi::dr_safe_write(
            target_ptr,
            to_write.len(),
            to_write.as_mut_ptr() as *mut c_void,
            &mut bytes_written,
        )
        .as_bool();
        if success {
            Ok(())
        } else {
            Err(WriteError {
                n_bytes_tried: to_write.len(),
                data_tried: to_write,
                n_bytes_written: bytes_written,
            })
        }
    }
}

/// Get name of the module, assuming valid utf8.
pub fn utf8_name_of_module(m: module_data_t) -> Result<String, Utf8NameError> {
    unsafe {
        let mod_name_ptr = ffi::dr_module_preferred_name(&m);
        if mod_name_ptr.is_null() {
            return Err(Utf8NameError::NullPtr);
        } else {
            let cstr_mod_name = CStr::from_ptr(mod_name_ptr);
            match cstr_mod_name.to_str() {
                Ok(v) => Ok(v.to_string()),
                Err(_) => Err(Utf8NameError::Malformed(
                    cstr_mod_name.to_string_lossy().to_string(),
                )),
            }
        }
    }
}

pub unsafe fn get_proc_address(base_addr: app_pc, name: &str) -> generic_func_t {
    unsafe {
        ffi::dr_get_proc_address(
            base_addr as *mut _module_handle_t,
            CString::new(name).unwrap().as_ptr(),
        )
    }
}
