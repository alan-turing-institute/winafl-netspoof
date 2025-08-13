use libc::{c_char, c_uchar, c_uint, c_void, free, malloc, memcpy};

pub type CommonFuzzStuffFn =
    Option<unsafe extern "C" fn(argv: *mut *mut c_char, buf: *mut c_uchar, len: c_uint) -> c_uchar>;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dll_mutate_testcase(
    argv: *mut *mut c_char,
    buf: *mut c_uchar,
    len: c_uint,
    common_fuzz_stuff: CommonFuzzStuffFn,
) -> c_uchar {
    unsafe {
        let mut bailout: c_uchar = 1;

        if buf.is_null() || len == 0 {
            return bailout;
        }

        // allocate new buffer
        let newbuf = malloc(len as usize) as *mut c_uchar;
        if newbuf.is_null() {
            return bailout;
        }

        // copy original
        memcpy(newbuf as *mut c_void, buf as *const c_void, len as usize);

        // mutate each byte
        for i in 0..len {
            *newbuf.add(i as usize) = (*newbuf.add(i as usize)).wrapping_add(1);

            if let Some(common_fuzz) = common_fuzz_stuff {
                if common_fuzz(argv, newbuf, len) != 0 {
                    bailout = 1;
                    break;
                }
            }
        }

        free(newbuf as *mut c_void);
        bailout
    }
}
