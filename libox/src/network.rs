use crate::drcore::log;
use std::{
    env,
    fmt::Display,
    fs::{File, metadata},
    io::Write,
    net::SocketAddr,
    sync::{LazyLock, Mutex},
};

static PCAP: LazyLock<Mutex<Vec<Packet>>> = LazyLock::new(|| Mutex::new(Vec::with_capacity(100)));

#[derive(Clone, Debug)]
pub struct PacketMeta {
    addr: SocketAddr,
    payload: Vec<u8>,
}

#[derive(Clone, Debug)]
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

fn is_magic_pkt(pkt: &Packet) -> bool {
    if let Packet::Inbound(PacketMeta { payload, .. }) = pkt {
        return payload.as_slice() == &[87, 111, 114, 108, 100];
    }
    false
}

pub fn push(pkt: Packet) {
    if is_magic_pkt(&pkt) {
        PCAP.lock().expect("Failed getting lock on PCAP").push(pkt);
        dump_pcap();
        panic!("found magic packet!");
    }
    PCAP.lock().expect("Failed getting lock on PCAP").push(pkt);
}

#[unsafe(no_mangle)]
pub extern "C" fn dump_pcap() -> i32 {
    match dump(DumpFormat::CSV) {
        Ok(()) => 0,
        Err(err_msg) => {
            log(&format!("Error dumping pcap as csv: {}\n", &err_msg));
            1
        }
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
                .write("src,dst,payload\n".as_bytes())
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
