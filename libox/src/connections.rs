use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{LazyLock, Mutex},
};

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
        .expect("failed getting lock on SOCKET_MAP")
        .insert(socket_id, Connection::new(socket_addr));
}

pub fn record_request(socket_id: usize, request: Vec<u8>) {
    CONNECTION_MAP
        .lock()
        .expect("failed getting lock on SOCKET_MAP")
        .get_mut(&socket_id)
        .expect("tried recording a request on an unconnected socket")
        .pending_request = Some(request);
}

pub fn get(socket_id: usize) -> Option<Connection> {
    CONNECTION_MAP
        .lock()
        .expect("failed getting lock on SOCKET_MAP")
        .get(&socket_id)
        .map(Connection::clone)
}
