use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{LazyLock, Mutex},
};

use crate::network::{self, Packet};

/// A spoofed network connection with the most recent request (sent by the binary over "send").
#[derive(Debug, Clone)]
pub struct Connection {
    pub addr: SocketAddr,
    pub pending_request: Option<Vec<u8>>,
}

impl ToString for Connection {
    fn to_string(&self) -> String {
        self.addr.to_string()
    }
}

impl Connection {
    pub fn new(socket_addr: SocketAddr) -> Self {
        Connection {
            addr: socket_addr,
            pending_request: None,
        }
    }
}

/// HashMap of currently open network connections.
static CONNECTION_MAP: LazyLock<Mutex<HashMap<usize, Connection>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn insert(socket_id: usize, socket_addr: SocketAddr) {
    CONNECTION_MAP
        .lock()
        .expect("failed getting lock on CONNECTION_MAP")
        .insert(socket_id, Connection::new(socket_addr));
}

pub fn record_request(socket_id: usize, request: Vec<u8>) {
    let mut connection_map = CONNECTION_MAP
        .lock()
        .expect("failed getting lock on CONNECTION_MAP");

    let connection = connection_map
        .get_mut(&socket_id)
        .expect("tried recording a request on an unconnected socket");

    // Cache most recent request against the connection.
    (*connection).pending_request = Some(request.clone());

    // Record the reqest with the network module.
    network::push(Packet::outbound(connection.addr, request));
}

pub fn record_response(socket_id: usize, response: Vec<u8>) {
    let src_addr = CONNECTION_MAP
        .lock()
        .expect("failed getting lock on CONNECTION_MAP")
        .get(&socket_id)
        .expect("tried recording a request on an unconnected socket")
        .addr
        .clone();

    // Record the response with the network module.
    network::push(Packet::inbound(src_addr, response));
}

pub fn get(socket_id: usize) -> Option<Connection> {
    CONNECTION_MAP
        .lock()
        .expect("failed getting lock on SOCKET_MAP")
        .get(&socket_id)
        .map(Connection::clone)
}
