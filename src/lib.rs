//!# Rhook
//!
//!Hooks libc functions with an easy API
//!
//!## Example
//!
//!Say you want to limit the badnwhidth of a program
//!
//!Usually downloading calls `libc::recv` function
//!
//!So our goal is to throttle it with a simple sleep
//!
//!To do that with this crate: (taking speedtest program as an example)
//!
//!1- Check its manpage https://man7.org/linux/man-pages/man2/recv.2.html to see what the
//!function's input/output
//!
//!2- use this crate
//!```rust
//!run_with(vec!("speedtest"), vec!(Hook::Recv(stringify!(|sockfd, buf, len, flags|{
//!  std::thread::sleep_ms(100);
//!  original_recv(sockfd, buf, len, flags)
//!}))))
//!```
//!
//!Thats it!
//!Note that you have acess inside the closure to the original function denoted by the prefix
//!`original_` + the function name
//!
//!
//!Couple of points:
//!- If you take ownership of an input value inside of the closure, be sure to use ManuallyDrop so
//!you don't free it
//!
//!Check out the tests for more examples

mod libc;
use std::io::Write;

use std::process::{Command, Stdio};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// libc hook
pub enum Hook {
    /// fn open(path: *const c_char, oflags: c_int) -> Option<c_int>,
    Open(&'static str),
    /// fn opendir(dirname: *const c_char) -> *mut DIR
    OpenDir(&'static str),
    /// fn recv( socket: c_int, buf: *mut c_void, len: size_t, flags: c_int,) -> ssize_t
    Recv(&'static str),
    /// fn read( fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t
    Read(&'static str),
}

/// run a program with the specified hooks
pub fn run_with(program: Vec<&str>, hooks: Vec<Hook>) -> Result<()> {
    prepare()?;
    for hook in hooks {
        match hook {
            Hook::Open(fun) => {
                append(libc::open(fun))?;
            }
            Hook::OpenDir(fun) => {
                append(libc::opendir(fun))?;
            }
            Hook::Recv(fun) => {
                append(libc::recv(fun))?;
            }
            Hook::Read(fun) => {
                append(libc::read(fun))?;
            }
        }
    }
    build_dylib()?;
    run_program(program)?;
    Ok(())
}

fn prepare() -> Result<()> {
    // Ignore project already exists error
    Command::new("cargo")
        .arg("new")
        .arg("botdyl")
        .arg("--lib")
        .current_dir("/tmp")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait()?;
    std::fs::write(
        "/tmp/botdyl/Cargo.toml",
        r#"[package]
name = "botdyl"
version = "0.1.0"
[lib]
crate-type = ["dylib"]
"#,
    )?;
    std::fs::write("/tmp/botdyl/src/lib.rs", include_str!("scaffold.rs"))?;
    Ok(())
}

fn append(fun: String) -> Result<()> {
    std::fs::OpenOptions::new()
        .append(true)
        .open("/tmp/botdyl/src/lib.rs")?
        .write_all(fun.as_bytes())?;
    Ok(())
}

fn build_dylib() -> Result<()> {
    let status = Command::new("cargo")
        .arg("b")
        .current_dir("/tmp/botdyl")
        .env("CARGO_TARGET_DIR", "/tmp/botdyl/target")
        .spawn()?
        .wait()?;
    if status.success() {
        Ok(())
    } else {
        Err("Failed to compile botdyl".into())
    }
}

fn run_program(p: Vec<&str>) -> Result<()> {
    Command::new(p[0])
        .args(&p[1..])
        .env("LD_PRELOAD", "/tmp/botdyl/target/debug/libbotdyl.so")
        .spawn()?
        .wait()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // see what file it acess
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
