use rhook::*;
use std::process::Command;

// return fake data for read
fn fake_cat() {
    Command::new("cat")
        .arg("Cargo.toml")
        .add_hooks(vec![
            Hook::Read(stringify!(|fd, buf, count| {
                let buf = buf as *mut u8;
                use std::io::Write;
                let mut buf = ManuallyDrop::new(std::slice::from_raw_parts_mut(buf, count));
                let msg = b"hello world";
                buf.write_all(msg);
                COUNTER += 1;
                if COUNTER % 2 != 0 {
                    msg.len() as isize
                } else {
                    0
                }
            })),
            Hook::Open(stringify!(|path, flags| {
                let path_name = ManuallyDrop::new(CString::from_raw(path as *mut _));
                original_open(path, flags)
            })),
        ])
        .set_hooks()
        .unwrap()
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn main() {
    fake_cat();
}
