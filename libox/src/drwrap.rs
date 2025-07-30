use crate::ffi;
use crate::utils::Boolean;
use std::os::raw::{c_int, c_void};

/// Interface for casting types to C void pointers (`void*).
pub trait VoidPtr {
    fn to_void_ptr(self) -> *mut c_void;
}

impl VoidPtr for Option<usize> {
    fn to_void_ptr(self) -> *mut c_void {
        match self {
            Some(v) => v as *mut c_void,
            None => std::ptr::null_mut(),
        }
    }
}

/// Sets the return value for a function that is being wrapped with the dynamorio drwrap extension.
/// May only be called from within a drwrap-pre or drwrap-post callback.
pub fn set_retval(wrapctx: *mut c_void, retval: Option<usize>) -> bool {
    unsafe { ffi::drwrap_set_retval(wrapctx, retval.to_void_ptr()).as_bool() }
}

/// May only be called from a \p drwrap_wrap pre-function callback.
/// Skips execution of the original function and returns to the function's caller with a return value of \p retval.
///
/// The post-function callback will not be invoked; nor will any pre-function callbacks (if multiple were registered) that have not yet been called. If the original function uses the "stdcall" calling convention, the total size of its arguments must be supplied. The return value is set regardless of whether the original function officially returns a value or not. Further state changes may be made with drwrap_get_mcontext() and drwrap_set_mcontext() prior to calling this function.
///
/// Note: It is up to the client to ensure that the application behaves as desired when the original function is skipped.
///
/// Returns whether successful.
pub fn skip_call(wrapctx: *mut c_void, retval: Option<usize>, stdcall_args_size: usize) -> bool {
    unsafe { ffi::drwrap_skip_call(wrapctx, retval.to_void_ptr(), stdcall_args_size).as_bool() }
}

pub fn get_arg(wrapctx: *mut c_void, idx: u8) -> *mut c_void {
    unsafe { ffi::drwrap_get_arg(wrapctx, idx as c_int) }
}
