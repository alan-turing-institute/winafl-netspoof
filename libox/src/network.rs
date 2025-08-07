use std::{
    env,
    fmt::Display,
    fs::{File, metadata},
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

impl Packet {
    pub fn to_csv(&self) -> String {
        match self {
            Packet::Inbound(PacketMeta { addr, payload }) => format!("{addr},self,{:?}\n", payload),
            Packet::Outbound(PacketMeta { addr, payload }) => {
                format!("self,{addr},{:?}\n", payload)
            }
        }
    }
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
pub extern "C" fn dump_pcap() -> i32 {
    match dump(DumpFormat::CSV) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

pub enum DumpFormat {
    CSV,
    HumanReadable,
}

pub fn dump(dump_fmt: DumpFormat) -> Result<(), String> {
    let pcap = PCAP
        .lock()
        .map_err(|e| format!("Failed getting lock on PCAP: {e}\n"))?;
    let out_file_path = env::var("PCAP").map_err(|e| format!("PCAP must be set as an environment variable, with a file path to the file used to dump captured network traffic: {e}\n"))?;

    let file_exists = metadata(&out_file_path).is_ok();

    let mut out_file = File::options()
        .create(true)
        .append(true)
        .open(&out_file_path)
        .map_err(|e| format!("Failed to open file at path {out_file_path}: {e}\n"))?;

    if !file_exists {
        if let DumpFormat::CSV = dump_fmt {
            out_file
                .write("src,dst,payload".as_bytes())
                .map_err(|e| format!("Failed writing csv header to file {out_file_path}: {e}\n"))?;
        }
    }

    for pkt in pcap.iter() {
        let serialise = match dump_fmt {
            DumpFormat::HumanReadable => Packet::to_string,
            DumpFormat::CSV => Packet::to_csv,
        };
        out_file
            .write_all(serialise(pkt).as_bytes())
            .map_err(|e| format!("Failed writing captured network packets to file: {e}\n"))?;
    }
    Ok(())
}
