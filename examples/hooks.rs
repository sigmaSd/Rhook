use rhook::*;
use std::process::Command;
fn main() {
    // see what file it access
    fn cat() {
        Command::new("cat")
            .arg("Cargo.toml")
            .add_hook(Hook::open(stringify!(|| {
                let path_name = ManuallyDrop::new(CString::from_raw(path as *mut _));
                dbg!(&path_name);
                None
            })))
            .set_hooks()
            .unwrap()
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    // see what directories it opens
    fn ls() {
        Command::new("ls")
            .arg("-l")
            .add_hook(Hook::opendir(stringify!(|| {
                dbg!(&ManuallyDrop::new(CString::from_raw(dirname as _)));
                None
            })))
            .set_hooks()
            .unwrap()
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    // limit bandwidth
    fn speedtest() {
        Command::new("speedtest")
            .add_hook(Hook::recv(stringify!(|| {
                std::thread::sleep(std::time::Duration::from_millis(180));
                None
            })))
            .set_hooks()
            .unwrap()
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    fn read_cat_data() {
        Command::new("cat")
            .arg("Cargo.toml")
            .add_hooks(vec![
                Hook::read(stringify!(|| {
                    let mut b = vec![0; count];
                    let n = original_read(fd, b.as_mut_ptr() as _, count);

                    // read the data
                    dbg!(String::from_utf8(b[..n as usize].to_vec()));

                    // write the data back
                    use std::io::Write;
                    let buf = buf as *mut u8;
                    let mut buf = ManuallyDrop::new(std::slice::from_raw_parts_mut(buf, count));
                    buf.write_all(&b[..n as usize]).unwrap();

                    Some(n)
                })),
                Hook::open(stringify!(|| {
                    let path_name = ManuallyDrop::new(CString::from_raw(path as *mut _));
                    dbg!(&path_name);
                    None
                })),
            ])
            .set_hooks()
            .map_err(|e| println!("{}", e))
            .unwrap()
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    cat();
    read_cat_data();
    ls();
    speedtest();
}
