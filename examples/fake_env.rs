use rhook::*;
use std::process::Command;
// Change the value read from the environment variable
fn fake_env() {
    Command::new("ls")
        .arg("-l")
        .add_hook(Hook::getenv(stringify!(|| {
            let env = ManuallyDrop::new(CString::from_raw(s as _));
            dbg!(&env);
            if env.to_str() == Ok("QUOTING_STYLE") {
                Some(CString::new("literal").unwrap().into_raw())
            } else if env.to_str() == Ok("TIME_STYLE") {
                Some(CString::new("locale").unwrap().into_raw())
            } else {
                None
            }
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
