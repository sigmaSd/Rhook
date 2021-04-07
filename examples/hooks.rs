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
                dbg!(&ManuallyDrop::new(CString::from_raw(
                    dirname as _
                )));
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

    // return fake data for read
    // not yet working, still figuring this out
    fn fake_cat() {
        Command::new("cat")
            .arg("Cargo.toml")
            .add_hooks(vec![
                Hook::read(stringify!(|| {
                    let mut b = vec![0; count];
                    let n = original_read(fd, b.as_mut_ptr() as _, count);
                    let mut buf: ManuallyDrop<&mut [u8]> =
                        ManuallyDrop::new(transmute(std::slice::from_raw_parts_mut(buf, count)));

                    *buf = &mut b"hello world qsdsds sqd qsqsdsq qs dqsd q".to_vec();
                    Some(n as isize)
                })),
                Hook::open(stringify!(|| {
                    let path_name = ManuallyDrop::new(CString::from_raw(path as *mut _));
                    dbg!(&path_name);
                    None
                })),
            ])
            .set_hooks()
            .unwrap()
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    cat();
    fake_cat();
    ls();
    speedtest();
}
