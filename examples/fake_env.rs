use rhook::*;
use std::process::Command;
// Change the value read from the environment variable
fn fake_env() {
    Command::new("ls")
        .arg("-l")
        .add_hook(Hook::GetEnv(stringify!(|envp| {
            use std::mem::ManuallyDrop;
            let env = ManuallyDrop::new(std::ffi::CString::from_raw(envp as _));
            dbg!(&env);
            if env.to_str() == Ok("QUOTING_STYLE") {
                return original_getenv(envp);
                std::ffi::CString::new("literal").unwrap().into_raw()
            } else if env.to_str() == Ok("TIME_STYLE") {
                std::ffi::CString::new("locale").unwrap().into_raw()
            } else {
                std::ffi::CString::new("").unwrap().into_raw()
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
