#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]

use libc::*;
use std::ffi::CString;
use std::mem::transmute;
use std::mem::ManuallyDrop;
