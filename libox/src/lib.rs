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
