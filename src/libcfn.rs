macro_rules! libc {
(fn $name: ident($($arg: ident: $typee: ty$(,)?)*) -> $ret_type: ty) => (
    pub(crate) fn $name(fun: &str) -> String {
        format!("{function_type}
            {{
            let original_{function_name} = dlsym(RTLD_NEXT, \"{function_name}\0\".as_ptr() as _);
            let original_{function_name}: {function_type_without_vars} = transmute(original_{function_name});
            ({user_closure}){function_vars} 
            }}
            "
        ,function_type = stringify!(#[no_mangle] pub unsafe extern "C" fn $name ($($arg: $typee,)*) -> $ret_type).to_string()
        ,function_type_without_vars = stringify!(extern "C" fn($($typee,)*) -> $ret_type)
        ,function_vars = stringify!(($($arg,)*))
        ,function_name = stringify!($name)
        ,user_closure = fun)})
}

// libc functions starts here

libc!(fn open(path: *const c_char, oflag: c_int) -> c_int);
libc!(fn opendir(dirname: *const c_char) -> *mut DIR);
libc!(fn recv(
            socket: c_int,
            buf: *mut c_void,
            len: size_t,
            flags: c_int,
        ) -> ssize_t);
libc!(fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t);
libc!(fn recv_msg(fd: c_int, msg: *mut msghdr, flags: c_int) -> ssize_t);
