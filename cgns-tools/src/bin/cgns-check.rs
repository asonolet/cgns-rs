//! CGNS file checker — `cgns-check [options] <file>`
//!
//! Checks a CGNS file for validity and prints summary statistics.
//! Compiled from the upstream CGNS `cgnscheck.c`.

macro_rules! tool_main {
    ($name:ident, $c_main:ident) => {
        use std::ffi::CString;
        use std::ptr;

        extern "C" {
            fn $c_main(argc: i32, argv: *const *const i8) -> i32;
        }

        fn main() {
            let args: Vec<CString> = std::env::args()
                .map(|a| CString::new(a).expect("args should not contain null"))
                .collect();
            let mut argv: Vec<*const i8> = args.iter().map(|a| a.as_ptr()).collect();
            argv.push(ptr::null());
            let code = unsafe { $c_main(args.len() as i32, argv.as_ptr()) };
            std::process::exit(code);
        }
    };
}

tool_main!(main, cgnscheck_main);
