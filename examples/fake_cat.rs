use rhook::*;
use std::process::Command;

// return fake data for read
fn fake_cat() {
    Command::new("cat")
        .arg("Cargo.toml")
        .add_hooks(vec![
            Hook::read(stringify!(|| {
                let buf = buf as *mut u8;
                use std::io::Write;
                let mut buf = ManuallyDrop::new(std::slice::from_raw_parts_mut(buf, count));
                let msg = b"hello world";
                buf.write_all(msg).unwrap();
                COUNTER += 1;

                if COUNTER % 2 != 0 {
                    Some(msg.len() as isize)
                } else {
                    Some(0)
                }
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

fn main() {
    fake_cat();
}
