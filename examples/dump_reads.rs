use rhook::*;

fn main() {
    std::process::Command::new("rust-analyzer")
        .add_hook(Hook::read(stringify!(|| {
            let mut b = vec![0; count];
            let n = original_read(fd, b.as_mut_ptr() as _, count);

            use std::io::Write;
            let mut log = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("./rhook_log")
                .unwrap();
            log.write_all(&b[..n as usize]).unwrap();

            let buf = buf as *mut u8;
            let mut buf = ManuallyDrop::new(std::slice::from_raw_parts_mut(buf, count));
            buf.write_all(&b[..n as usize]).unwrap();

            Some(n)
        })))
        .set_hooks()
        .map_err(|e| println!("{}", e))
        .unwrap()
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
