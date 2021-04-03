#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::mem::transmute;
use std::os::raw::c_void;

// definitions taken from rust libc crate
pub type c_char = i8;
pub type c_int = i32;
pub type CPtr = *const u8;
//pub type c_long = i64;
pub type ssize_t = isize;
pub type size_t = usize;
pub enum DIR {}

extern "C" {
    fn dlsym(handle: CPtr, symbol: CPtr) -> CPtr;
}
