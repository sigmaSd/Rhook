//!# Rhook
//!
//!Hook libc functions with an easy API
//!
//!## Usage
//!
//!1- Import the trait [RunHook]
//!
//!2- Create an [Command](std::process::Command) with [Command::new](std::process::Command::new) and add hooks to it via [add_hook](RunHook::add_hook) and [add_hooks](RunHook::add_hooks) methods
//!
//!3- Confirm the hooks with [set_hooks](RunHook::set_hooks) method this step is necessary
//!
//!4- Now you can carry on with the usual [Command](std::process::Command) methods ([output](std::process::Command::output), [spawn](std::process::Command::spawn),[status](std::process::Command::status),..)
//!
//!## Example
//!
//!Say you want to limit the bandwidth of a program
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
//!use rhook::{RunHook, Hook};
//!
//!std::process::Command::new("speedtest").add_hook(Hook::Recv(stringify!(|sockfd, buf, len, flags|{
//!  std::thread::sleep_ms(10);
//!  original_recv(sockfd, buf, len, flags)
//!}))).set_hooks().unwrap().spawn();
//!```
//!
//!Thats it!
//!Note that you have acess inside the closure to the original function denoted by the prefix
//!`original_` + the function name
//!
//!Couple of points:
//!- If you take ownership of an input value inside of the closure, be sure to use ManuallyDrop so
//!you don't free it
//!
//!Check out the tests for more examples

#[cfg(not(unix))]
compile_error!("This crate is unix only");

mod libc;
use std::{
    collections::HashSet,
    io::{self, Write},
    sync::Mutex,
};

use once_cell::sync::Lazy;
use std::io::Read;
use std::process::{Command, Stdio};

//type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
use std::io::Result;

static HOOKS: Lazy<Mutex<HashSet<Hook>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// libc hooks enum
#[non_exhaustive]
#[derive(Hash, Eq, PartialEq)]
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

/// Specify libc hooks for a Command
pub trait RunHook {
    /// Add a libc hook to the command
    fn add_hook(&mut self, hook: Hook) -> &mut Self {
        HOOKS.lock().expect("There is only one thread").insert(hook);
        self
    }
    /// Add a Vec of libc hooks to the command
    fn add_hooks(&mut self, hooks: Vec<Hook>) -> &mut Self {
        for hook in hooks {
            self.add_hook(hook);
        }
        self
    }
    /// Set the hooks, this is a required method since it does the actual work of creating a
    /// dynamic library and linking the target program with it
    fn set_hooks(&mut self) -> Result<&mut Self>;
}

impl RunHook for Command {
    fn set_hooks(&mut self) -> Result<&mut Self> {
        prepare()?;
        for hook in HOOKS.lock().expect("There is only one thread").drain() {
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

        Ok(self.env("LD_PRELOAD", "/tmp/rhookdyl/target/debug/librhookdyl.so"))
    }
}

/// Create the dynamic library and write the scaffold to it
fn prepare() -> Result<()> {
    const CARGO_TOML: &str = r#"[package]
name = "rhookdyl"
version = "0.1.0"
[lib]
crate-type = ["dylib"]"#;

    // Ignore project already exists error
    Command::new("cargo")
        .arg("new")
        .arg("rhookdyl")
        .arg("--lib")
        .current_dir("/tmp")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait()?;
    std::fs::write("/tmp/rhookdyl/Cargo.toml", CARGO_TOML)?;
    std::fs::write("/tmp/rhookdyl/src/lib.rs", include_str!("scaffold.rs"))?;
    Ok(())
}

/// Append rust generated code to the initial scaffold
fn append(fun: String) -> Result<()> {
    std::fs::OpenOptions::new()
        .append(true)
        .open("/tmp/rhookdyl/src/lib.rs")?
        .write_all(fun.as_bytes())?;
    Ok(())
}

/// Build the dynamic library
fn build_dylib() -> Result<()> {
    let mut process = Command::new("cargo")
        .arg("b")
        .current_dir("/tmp/rhookdyl")
        .env("CARGO_TARGET_DIR", "/tmp/rhookdyl/target")
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stderr = String::new();
    process
        .stderr
        .as_mut()
        .expect("stderr is piped")
        .read_to_string(&mut stderr)?;

    let status = process.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, stderr))
    }
}
