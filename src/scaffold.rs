#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]

use libc::*;
use std::cell::RefCell;
use std::ffi::CString;
use std::mem::transmute;
use std::mem::ManuallyDrop;

thread_local! {
    static COUNTER: RefCell<isize> = RefCell::new(0);
}
