use rhook::*;
use std::process::Command;

// return fake data for read
fn multithread() {
    macro_rules! command {
        ($msg: expr) => {
            Command::new("cat")
                .arg("Cargo.toml")
                .add_hook(Hook::read(stringify!(|| {
                    let buf = buf as *mut u8;
                    use std::io::Write;
                    let mut buf = ManuallyDrop::new(std::slice::from_raw_parts_mut(buf, count));
                    let msg = $msg;
                    let msg = msg.as_bytes();
                    buf.write_all(msg).unwrap();
                    COUNTER += 1;

                    if COUNTER % 2 != 0 {
                        Some(msg.len() as isize)
                    } else {
                        Some(0)
                    }
                })))
                .set_hooks()
                .unwrap()
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        };
    }
    let t1 = std::thread::spawn(|| command!("hello"));
    let t2 = std::thread::spawn(|| command!("bye"));
    t1.join().unwrap();
    t2.join().unwrap();
}

fn main() {
    multithread();
}
