use std::{
    env,
    fmt::Display,
    fs::File,
    io::Write,
    net::SocketAddr,
    sync::{LazyLock, Mutex},
};

static PCAP: LazyLock<Mutex<Vec<Packet>>> = LazyLock::new(|| Mutex::new(Vec::with_capacity(100)));

pub struct PacketMeta {
    addr: SocketAddr,
    payload: Vec<u8>,
}

pub enum Packet {
    Inbound(PacketMeta),
    Outbound(PacketMeta),
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Packet::Inbound(PacketMeta { addr, payload }) => {
                write!(f, "{addr} -> self {:?}\n", payload)?
            }
            Packet::Outbound(PacketMeta { addr, payload }) => {
                write!(f, "self -> {addr} {:?}\n", payload)?
            }
        }
        Ok(())
    }
}

impl Packet {
    pub fn inbound(src: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::Inbound(PacketMeta { addr: src, payload })
    }
    pub fn outbound(dst: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::Outbound(PacketMeta { addr: dst, payload })
    }
}

pub fn push(pkt: Packet) {
    PCAP.lock().expect("Failed getting lock on PCAP").push(pkt)
}

#[unsafe(no_mangle)]
pub extern "C" fn dump_pcap() {
    dump();
}

pub fn dump() {
    let pcap = PCAP.lock().expect("Failed getting lock on PCAP");
    let out_file_path = env::var("PCAP").expect("PCAP must be set as an environment variable, with a file path to the file used to dump captured network traffic.");

    let mut out_file = File::options()
        .append(true)
        .open(out_file_path)
        .expect("Failed to open file at path: {out_file_path}");

    for pkt in pcap.iter() {
        out_file
            .write_all(pkt.to_string().as_bytes())
            .expect("Failed writing captured network packets to file.");
    }
}
