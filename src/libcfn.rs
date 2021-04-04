pub(crate) fn open(fun: &str) -> String {
    stringify!(
        #[no_mangle]
        pub unsafe extern "C" fn open(path: *const c_char, oflag: c_int) -> c_int {
            let original_open = dlsym(RTLD_NEXT, "open\0".as_ptr() as _);
            let original_open: extern "C" fn(*const c_char, c_int) -> c_int =
                transmute(original_open);
        }
    )
    .remove_last_char()
        + &format!("({})(path, oflag)}}", fun)
}

pub(crate) fn opendir(fun: &str) -> String {
    stringify!(
        #[no_mangle]
        pub unsafe extern "C" fn opendir(dirname: *const c_char) -> *mut DIR {
            let original_opendir = dlsym(RTLD_NEXT, "opendir\0".as_ptr() as _);
            let original_opendir: extern "C" fn(*const c_char) -> *mut DIR =
                transmute(original_opendir);
        }
    )
    .remove_last_char()
        + &format!("({})(dirname)}}", fun)
}

pub(crate) fn recv(fun: &str) -> String {
    stringify!(
        #[no_mangle]
        pub unsafe extern "C" fn recv(
            socket: c_int,
            buf: *mut c_void,
            len: size_t,
            flags: c_int,
        ) -> ssize_t {
            let original_recv = dlsym(RTLD_NEXT, "recv\0".as_ptr() as _);
            let original_recv: extern "C" fn(c_int, *mut c_void, size_t, c_int) -> ssize_t =
                transmute(original_recv);
        }
    )
    .remove_last_char()
        + &format!("({})(socket, buf, len, flags)}}", fun)
}

pub(crate) fn read(fun: &str) -> String {
    stringify!(
        #[no_mangle]
        pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
            let original_read = dlsym(RTLD_NEXT, "read\0".as_ptr() as _);
            let original_read: extern "C" fn(c_int, *mut c_void, size_t) -> ssize_t =
                transmute(original_read);
        }
    )
    .remove_last_char()
        + &format!("({})(fd,buf,count)}}", fun)
}

pub(crate) fn recv_msg(fun: &str) -> String {
    stringify!(
        #[no_mangle]
        pub unsafe extern "C" fn recv_msg(fd: c_int, msg: *mut msghdr, flags: c_int) -> ssize_t {
            let original_recvmsg = dlsym(RTLD_NEXT, "recv_msg\0".as_ptr() as _);
            let original_recvmsg: extern "C" fn(c_int, *mut msghdr, c_int) -> ssize_t =
                transmute(original_recvmsg);
        }
    )
    .remove_last_char()
        + &format!("({})(fd,msg,flags)}}", fun)
}

// helper
trait StringTools {
    /// for stringify to work we add a }
    /// this method gets rid of that char + convert the str to string
    fn remove_last_char(self) -> String;
}

impl StringTools for &str {
    fn remove_last_char(self) -> String {
        let mut string = self.to_string();
        string.pop();
        string
    }
}
