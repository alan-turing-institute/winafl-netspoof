use crate::drcore::log;
use std::env;
use std::fs::read;

pub fn respond(request: Vec<u8>) -> Vec<u8> {
    let input_file_path = env::var("FUZZ_INPUT").expect("FUZZ_INPUT must be set as an environment variable, with a file path to the fuzz input file: the file that AFL will write the mutated fuzz input to.");

    log(&format!("request={:?}", request));

    let response = read(input_file_path).unwrap();

    log(&format!("request={:?} -> response={:?}", request, response));
    response
}
