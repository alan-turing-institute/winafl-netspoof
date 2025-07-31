use crate::drcore::log;
use std::env;
use std::fs::File;
use std::io::Read;

pub fn respond(request: Vec<u8>) -> Vec<u8> {
    let input_file_path = env::var("FUZZ_INPUT").expect("FUZZ_INPUT must be set as an environment variable, with a file path to the fuzz input file: the file that AFL will write the mutated fuzz input to.");

    log(&format!("request={:?}", request));

    let mut f = File::open("foo.txt").expect(&format!(
        "Failed to read FUZZ_INPUT file at path: {}",
        input_file_path,
    ));

    let mut response = Vec::new();
    f.read_to_end(&mut response).expect(&format!(
        "Failed reading bytes from file: {}",
        input_file_path
    ));

    log(&format!("request={:?} -> response={:?}", request, response));
    response
}
