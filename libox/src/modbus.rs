use crate::drcore::log;
use crate::fuzzer;
use modbus_core::tcp::ResponseAdu;
use modbus_core::tcp::server::encode_response;
use modbus_core::tcp::{RequestAdu, server::decode_request};
use modbus_core::{Coils, Request, Response, ResponsePdu};

pub fn respond(request: Vec<u8>) -> Result<Vec<u8>, String> {
    let request_payload = decode_request(&request)
        .map_err(|e| format!("Failed to decode Modbus payload from TCP request: {e}"))?;

    // Parse the Request ADU.
    let RequestAdu { hdr, pdu } =
        request_payload.ok_or("Failed parsing request payload as Modbus".to_string())?;

    // Assemble a Response PDU by matching on the request type.
    let mut response_pdu_buf = vec![0u8; 256];
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
            // Initialise a coil array with dummy values.
            let mut coils = vec![false; n_coils];
            // Unpack the bytes from the fuzzer into the coil array.
            modbus_core::unpack_coils(&content, n_coils, &mut coils)
                .map_err(|e| format!("Failed to unpack coils: {e}"))?;

            response_pdu_buf.truncate(content_len);
            Ok(Response::ReadCoils(
                Coils::from_bools(&coils, &mut response_pdu_buf).unwrap(),
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

    log(&format!(
        "[modbus] RESPONSE PDU: {:?}",
        response_pdu.unwrap()
    ));
    let response_adu = ResponseAdu {
        hdr,
        pdu: ResponsePdu(response_pdu),
    };
    let mut response = vec![0u8; 256];
    encode_response(response_adu, &mut response)
        .map(|n_bytes_written| {
            response.truncate(n_bytes_written);
            response
        })
        .map_err(|e| format!("Failed encoding the response: {e}"))
}
