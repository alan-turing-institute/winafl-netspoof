use crate::connections;
use crate::drcore;
use crate::drcore::log;
use crate::drwrap;
use crate::responder;
use crate::utils::FromBuf;
use std::panic;
use std::{net::SocketAddr, os::raw::c_void};

/// Windows docs for "connect": https://learn.microsoft.com/en-us/windows/win32/api/Winsock2/nf-winsock2-connect
#[unsafe(no_mangle)]
pub extern "C" fn wrap_pre_connect(wrapctx: *mut c_void) {
    // Override the panic hook to log the DynamoRio logger when a panic occurs.
    panic::set_hook(Box::new(|panic_info| {
        log(&format!("panic occurred: {panic_info}"));
    }));

    /// Skip the call the wrapped function and set a "success" return value.
    fn clean_exit(wrapctx: *mut c_void) {
        match drwrap::skip_call(wrapctx, None, 0) {
            true => log("[connect] successfully returned NULL"),
            false => log("[connect] skip_call failed"),
        };
    }

    // Opaque handle for the socket (provided by the OS).
    let socket_id = drwrap::get_arg(wrapctx, 0) as usize;
    // Pointer to the sockaddr struct.
    let sockaddr_ptr = drwrap::get_arg(wrapctx, 1);
    // Size of the sockaddr struct in bytes.
    let sockaddr_size = drwrap::get_arg(wrapctx, 2) as usize;

    if sockaddr_ptr.is_null() || sockaddr_size == 0 {
        log("[connect] Invalid sockaddr pointer or size of sockaddr struct is 0");
        return clean_exit(wrapctx);
    }

    // Try to parse a std::net::SocketAddr from the pointer.
    let socket_addr = match drcore::safe_read(sockaddr_ptr, sockaddr_size) {
        Ok(buf) => SocketAddr::from_buf(buf),
        Err(read_err) => {
            log(&format!(
                "[connect] Failed reading sockaddr:\n {:?}",
                read_err
            ));
            None
        }
    };

    match socket_addr {
        None => return clean_exit(wrapctx),
        Some(addr) => {
            log(&format!(
                "[connect] attempt to connect to {}",
                addr.to_string()
            ));
            // Keep track of this connection.
            connections::insert(socket_id, addr);
        }
    };

    return clean_exit(wrapctx);
}

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
    let to_write = responder::respond(pending_requst);

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
