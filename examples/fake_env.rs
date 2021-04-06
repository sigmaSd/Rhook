use rhook::*;
use std::process::Command;
// Change the value read from the environment variable
fn fake_env() {
    Command::new("ls")
        .arg("QUOTING_STYLE")
        .add_hook(Hook::GetEnv(stringify!(|s| {
            dbg!(std::ffi::CString::from_raw(s as _));
            std::ffi::CString::new("literal").unwrap().into_raw()
        })))
        .set_hooks()
        .unwrap()
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn main() {
    fake_env();
}
