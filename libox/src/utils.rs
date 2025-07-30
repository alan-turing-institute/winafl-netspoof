use os_socketaddr::OsSocketAddr;
use winapi::shared::ws2def::SOCKADDR;

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
