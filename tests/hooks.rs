#[cfg(test)]
mod tests {
    use rhook::*;
    #[test]
    // see what file it access
    fn cat() {
        run_with(
            vec!["cat", "Cargo.toml"],
            vec![Hook::Open(stringify!(|path, flags| {
                use std::ffi::CString;
                use std::mem::ManuallyDrop;
                let path_name = ManuallyDrop::new(CString::from_raw(path as *mut _));
                dbg!(&path_name);
                original_open(path, flags)
            }))],
        )
        .unwrap();
    }

    #[test]
    // see what directories it opens
    fn ls() {
        run_with(
            vec!["ls", "-l"],
            vec![Hook::OpenDir(stringify!(|dirname| {
                dbg!(&std::mem::ManuallyDrop::new(std::ffi::CString::from_raw(
                    dirname as _
                )));
                original_opendir(dirname)
            }))],
        )
        .unwrap();
    }

    #[test]
    // limit bandwidth
    fn speedtest() {
        run_with(
            vec!["speedtest"],
            vec![Hook::Recv(stringify!(|socket, buf, len, flags| {
                std::thread::sleep(std::time::Duration::from_millis(180));
                original_recv(socket, buf, len, flags)
            }))],
        )
        .unwrap();
    }

    #[test]
    // return fake data for read
    // not yet working, still figuring this out
    fn fake_cat() {
        run_with(
            vec!["cat", "Cargo.toml"],
            vec![
                Hook::Read(stringify!(|fd, buf, count| {
                    use std::mem::ManuallyDrop;
                    let mut b = vec![0; count];
                    let n = original_read(fd, b.as_mut_ptr() as _, count);
                    let mut buf: ManuallyDrop<&mut [u8]> =
                        ManuallyDrop::new(transmute(std::slice::from_raw_parts_mut(buf, count)));

                    *buf = &mut b"hello world qsdsds sqd qsqsdsq qs dqsd q".to_vec();
                    n as isize
                })),
                Hook::Open(stringify!(|path, flags| {
                    use std::ffi::CString;
                    use std::mem::ManuallyDrop;
                    let path_name = ManuallyDrop::new(CString::from_raw(path as *mut _));
                    dbg!(&path_name);
                    original_open(path, flags)
                })),
            ],
        )
        .unwrap();
    }
}
