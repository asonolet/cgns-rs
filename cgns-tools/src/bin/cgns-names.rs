//! CGNS identifier lookup — `cgns-names [options] <pattern>`
//!
//! Looks up CGNS identifiers (names, types) matching a pattern.
//! Compiled from the upstream CGNS `cgnsnames.c`.

use std::ffi::CString;
use std::ptr;

extern "C" {
    fn cgnsnames_main(argc: i32, argv: *const *const i8) -> i32;
}

fn main() {
    let args: Vec<CString> = std::env::args()
        .map(|a| CString::new(a).expect("args should not contain null"))
        .collect();
    let mut argv: Vec<*const i8> = args.iter().map(|a| a.as_ptr()).collect();
    argv.push(ptr::null());
    let code = unsafe { cgnsnames_main(args.len() as i32, argv.as_ptr()) };
    std::process::exit(code);
}
