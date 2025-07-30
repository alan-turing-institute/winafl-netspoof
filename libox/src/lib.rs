mod connections;
mod drcore;
mod drwrap;
mod ffi;
mod modbus;
mod utils;
mod wrappers;

use drcore::log;
use std::{net::SocketAddr, os::raw::c_void};
use utils::FromBuf;

/// Windows docs for "send": https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-send
#[unsafe(no_mangle)]
pub extern "C" fn wrap_pre_send(wrapctx: *mut c_void, _user_data: *mut *mut c_void) {
    // Opaque handle for the socket (provided by the OS).
    let socket_id = drwrap::get_arg(wrapctx, 0) as usize;
    let payload_ptr = drwrap::get_arg(wrapctx, 1);
    let payload_size = drwrap::get_arg(wrapctx, 2) as usize;

    match connections::get(socket_id) {
        None => log(&format!("[send] tried to send to an unconnected socket!")),
        Some(addr) => {
            match drcore::safe_read(payload_ptr, payload_size) {
                Ok(payload) => {
                    log(&format!(
                        "[send] target: {}, payload: {:?}",
                        addr.to_string(),
                        payload
                    ));
                    connections::record_request(socket_id, payload);
                }
                Err(read_err) => {
                    log(&format!("[send] Failed reading payload: {:?}", read_err));
                }
            };
        }
    };

    // As specified in the "send" api docs.
    let retval = Some(payload_size);
    match drwrap::skip_call(wrapctx, retval, 0) {
        true => log(&format!(
            "[send] successfully returned with payload length ({payload_size})"
        )),
        false => log("[send] skip_call failed"),
    };
}

/// Windows docs for "recv": https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-recv
#[unsafe(no_mangle)]
pub extern "C" fn wrap_pre_recv(wrapctx: *mut c_void, _user_data: *mut *mut c_void) {
    // Opaque handle for the socket (provided by the OS).
    let socket_id = drwrap::get_arg(wrapctx, 0) as usize;
    let socket_addr = connections::get(socket_id);
    let socket_addr_str = match socket_addr.clone() {
        None => {
            log(&format!("[recv] called with an unconnected socket!"));
            "unknown"
        }
        Some(addr) => {
            log(&format!("[recv] called for: {}", addr.to_string()));
            &addr.to_string()
        }
    };

    // Pointer to the recv buffer that the exe will read from.
    let buf_ptr = drwrap::get_arg(wrapctx, 1);
    // Must write <= buf_size to buf_ptr.
    let buf_size = drwrap::get_arg(wrapctx, 2) as usize;

    let pending_requst = socket_addr
        .expect("need to have an existing connection to respond")
        .pending_request
        .expect("need a pending request to response");
    let to_write = modbus::respond(pending_requst).unwrap();

    assert!(to_write.len() <= buf_size);

    match drcore::safe_write(buf_ptr, to_write.clone()) {
        Ok(()) => {
            log(&format!(
                "[recv] target: {}, payload: {:?}",
                socket_addr_str,
                to_write.as_slice()
            ));
        }
        Err(write_err) => {
            log(&format!("[recv] Failed writing payload: {:?}", write_err));
        }
    };

    // As specified in the "recv" api docs.
    let retval = Some(to_write.len());
    match drwrap::skip_call(wrapctx, retval, 0) {
        true => log(&format!(
            "[recv] successfully returned with length of recv payload ({})",
            to_write.len()
        )),
        false => log("[recv] skip_call failed"),
    };
}
