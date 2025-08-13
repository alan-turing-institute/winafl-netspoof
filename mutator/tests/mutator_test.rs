use libloading::{Library, Symbol};
use std::{env, ffi::c_void, path::PathBuf};

type U8 = u8;
type U32 = u32;
type CChar = *mut i8;
type CommonFuzzStuffFn =
    Option<unsafe extern "C" fn(argv: *mut CChar, buf: *mut U8, len: U32) -> U8>;
type MutatorFn = unsafe extern "C" fn(
    argv: *mut CChar,
    buf: *mut U8,
    len: U32,
    common_fuzz_stuff: CommonFuzzStuffFn,
) -> U8;

unsafe extern "C" fn my_common_fuzz(_argv: *mut CChar, buf: *mut U8, len: U32) -> U8 {
    unsafe {
        let slice = std::slice::from_raw_parts(buf, len as usize);
        println!(
            "[common_fuzz_stuff] called with len={}, first byte={:?}",
            len, slice
        );
        0 // keep mutating
    }
}

#[test]
fn test_dll_mutate_testcase() {
    // Figure out path to compiled DLL
    let mut dll_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dll_path.push("target");
    dll_path.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    dll_path.push(if cfg!(target_os = "windows") {
        "mutator.dll"
    } else if cfg!(target_os = "linux") {
        "libmutator.so"
    } else {
        "libmutator.dylib"
    });

    assert!(
        dll_path.exists(),
        "DLL not found at {:?}. Did you run `cargo build` first?",
        dll_path
    );

    // Load DLL
    let lib = unsafe { Library::new(dll_path) }.expect("Failed to load mutator DLL");

    unsafe {
        // Get the function
        let func: Symbol<MutatorFn> = lib
            .get(b"dll_mutate_testcase")
            .expect("Failed to find symbol");

        let mut buf = [0u8, 1, 2, 3];
        println!("Before: {:?}", buf);

        let bailout = func(
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            buf.len() as u32,
            Some(my_common_fuzz),
        );

        println!("After call, bailout={}", bailout);
        println!("Original buffer after call: {:?}", buf);
    }
}
