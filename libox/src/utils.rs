use std::net::SocketAddr;

use os_socketaddr::OsSocketAddr;
use std::mem::{size_of, zeroed};
use winapi::shared::ws2def::{SOCKADDR, SOCKADDR_STORAGE};
use winapi::um::winsock2::getpeername;

pub trait Boolean {
    fn as_bool(self: Self) -> bool;
}

pub trait FromBuf {
    fn from_buf(buf: Vec<u8>) -> Option<Self>
    where
        Self: Sized;
}

impl FromBuf for std::net::SocketAddr {
    fn from_buf(buf: Vec<u8>) -> Option<Self> {
        let sockaddr_ptr = buf.as_ptr() as *const SOCKADDR;
        // Unsafe block is safe for all `from_buf` inputs, because `socketaddr_ptr` is
        // guranteed to be a pointer to a valid slice of length `buf.len()`.
        unsafe { OsSocketAddr::copy_from_raw(sockaddr_ptr, buf.len() as i32).into() }
    }
}

pub unsafe fn socketaddr_from_socket_id(socket_id: usize) -> Result<SocketAddr, String> {
    unsafe {
        let mut addr: SOCKADDR_STORAGE = zeroed();
        let mut len = size_of::<SOCKADDR_STORAGE>() as i32;

        match getpeername(socket_id, &mut addr as *mut _ as *mut SOCKADDR, &mut len) {
            0 => {
                let ptr = &addr as *const _ as *const SOCKADDR;
                let socketaddr: Option<SocketAddr> = OsSocketAddr::copy_from_raw(ptr, len).into();
                socketaddr.ok_or(String::from("Failed converting an os_socketaddr::OsSocketAddr to a std::net::SocketAddr, .sa_family must resolve to AF_INET or AF_INET6."))
            }
            _ => Err(String::from("getpeername returned a non-zero exit code")),
        }
    }
}

#[derive(Debug)]
pub struct ReadError {
    pub n_bytes_tried: usize,
    pub n_bytes_read: usize,
    pub buf: Vec<u8>,
}

#[derive(Debug)]
pub struct WriteError {
    pub data_tried: Vec<u8>,
    pub n_bytes_tried: usize,
    pub n_bytes_written: usize,
}

#[derive(Debug)]
pub enum Utf8NameError {
    NullPtr,
    Malformed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_name_error() {
        let e = Utf8NameError::Malformed(String::from("aMalformedModName"));
        println!("utf8 parse failed with: {:?}", e);
    }
}
