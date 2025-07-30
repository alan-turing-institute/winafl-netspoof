use crate::drcore::log;
use modbus_core::tcp::ResponseAdu;
use modbus_core::tcp::server::encode_response;
use modbus_core::tcp::{RequestAdu, server::decode_request};
use modbus_core::{Coils, Request, Response, ResponsePdu};

pub fn respond(request: Vec<u8>) -> Result<Vec<u8>, modbus_core::Error> {
    let mut buf = vec![0u8; 256];

    let mut response = vec![0u8; 256];

    match decode_request(&request) {
        Ok(data) => {
            let RequestAdu { hdr, pdu } = data.unwrap();
            let response_pdu = match pdu.0 {
                Request::ReadCoils(addr, n_coils) => {
                    log(&format!(
                        "[modbus] REQUEST: read {} coils @ 0x{:04X}",
                        n_coils, addr
                    ));
                    // 1 bit per coil, 8 coils per byte
                    buf.truncate(n_coils.div_ceil(8).into());
                    Ok(Response::ReadCoils(
                        Coils::from_bools(&vec![true; n_coils as usize], &mut buf).unwrap(),
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
            encode_response(response_adu, &mut response).map(|n_bytes_written| {
                response.truncate(n_bytes_written);
                response
            })
        }
        Err(_) => todo!("handle error decoding request"),
    }
}
