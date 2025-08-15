use crate::drcore::log;
use crate::fuzzer;
use modbus_core::tcp::ResponseAdu;
use modbus_core::tcp::server::encode_response;
use modbus_core::tcp::{RequestAdu, server::decode_request};
use modbus_core::{Coils, Request, Response, ResponsePdu};

pub fn respond(request: Vec<u8>) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; 256];

    let mut response = vec![0u8; 256];

    match decode_request(&request) {
        Ok(data) => {
            let RequestAdu { hdr, pdu } = data.expect("Failed parsing request payload as Modbus");
            let response_pdu = match pdu.0 {
                Request::ReadCoils(addr, n_coils) => {
                    log(&format!(
                        "[modbus] REQUEST: read {} coils @ 0x{:04X}",
                        n_coils, addr
                    ));
                    // 1 bit per coil, 8 coils per byte
                    let content_len: usize = n_coils.div_ceil(8).into();
                    let content = fuzzer::call();
                    if content.len() != content_len {
                        log(&format!(
                            "[modbus] Fuzzer content wasn't expect length of {content_len}!"
                        ));
                    }
                    let mut coils = Vec::new();
                    modbus_core::unpack_coils(&content, n_coils, &mut coils)
                        .map_err(|e| format!("Failed to unpack coils: {e}"))?;
                    buf.truncate(content_len);
                    Ok(Response::ReadCoils(
                        Coils::from_bools(&coils, &mut buf).unwrap(),
                    ))
                }
                Request::WriteSingleCoil(addr, coil) => {
                    log(&format!(
                        "[modbus] REQUEST: write single coil: ={} @ 0x{:04X}",
                        coil as u8, addr
                    ));
                    Ok(Response::WriteSingleCoil(addr))
                }
                _ => todo!("other request types"),
            };
            log(&format!("[modbus] RESPONSE: {:?}", response_pdu.unwrap()));
            let response_adu = ResponseAdu {
                hdr,
                pdu: ResponsePdu(response_pdu),
            };
            encode_response(response_adu, &mut response)
                .map(|n_bytes_written| {
                    response.truncate(n_bytes_written);
                    response
                })
                .map_err(|e| format!("Failed encoding the response: {e}"))
        }
        Err(_) => todo!("handle error decoding request"),
    }
}
