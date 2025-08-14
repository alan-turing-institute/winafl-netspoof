use std::{fs::OpenOptions, io::Write, slice};

use libc::{c_char, c_uchar, c_uint, c_void};

const NO_FUTHER_MUTATIONS: u8 = 1;

pub type WriteToTestCaseFn = extern "C" fn(*mut c_void, c_uint);
pub type RunTargetFn = extern "C" fn(*mut *mut c_char, c_uint) -> c_uchar;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dll_trim_testcase(
    _len: *mut c_uint,
    _exec_cksum: c_uint,
    _an_buf: *mut c_uchar,
    _trace_bits: *mut c_uchar,
    _write_to_testcase: WriteToTestCaseFn,
    _run_target: RunTargetFn,
    _argv: *mut *mut c_char,
    _exec_timeout: c_uint,
) -> c_uchar {
    // Indicate that no write is needed, buffer remains uncahnged.
    0
}

pub type CommonFuzzStuffFn =
    Option<unsafe extern "C" fn(argv: *mut *mut c_char, buf: *mut c_uchar, len: c_uint) -> c_uchar>;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dll_mutate_testcase_with_energy(
    argv: *mut *mut c_char,
    buf: *mut c_uchar,
    len: c_uint,
    energy: c_uint,
    common_fuzz_stuff: CommonFuzzStuffFn,
) -> c_uchar {
    unsafe {
        if buf.is_null() || len == 0 {
            return NO_FUTHER_MUTATIONS;
        };

        let slice = slice::from_raw_parts(buf, len as usize);

        let mut log_f = OpenOptions::new()
            .create(true)
            .append(true)
            .open("mutator_log.txt")
            .expect("Failed to create or open mutator_log.txt");

        writeln!(log_f, "{:?} has energy {}", slice, energy)
            .expect("Failed to write to mutation_log.txt file");

        let focus_len = match energy {
            // focus on the first 20% of the bytes when energy is low.
            0..256 => (0.2 * len as f32).floor() as usize,
            _ => len as usize,
        };

        // mutate each byte
        for i in 0..focus_len {
            let mut mutated = slice.to_owned();
            mutated[i] = mutated[i].wrapping_add(1);

            if let Some(common_fuzz) = common_fuzz_stuff {
                if common_fuzz(argv, mutated.as_ptr() as *mut u8, len) != 0 {
                    // signal from afl to stop mutating this input
                    break;
                }
            }
        }

        return NO_FUTHER_MUTATIONS;
    }
}
