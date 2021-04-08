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
//!3.1- Hooks are closures that takes no input and return an option of the libc function as output.
//!
//! If the closure return `None` that is equivalent to returning `Some(original_function(args))` in
//! other words it will run and use the original function output
//!
//! Inside the closure you have access to the libc function input + some imports from std (see
//! src/scaffold.rs)
//!
//!4- Now you can carry on with the usual [Command](std::process::Command) methods ([output](std::process::Command::output), [spawn](std::process::Command::spawn),[status](std::process::Command::status),..)
//!
//!
//!**Tricks:**
//!
//! The closure used for hooks have acess to many things: (imported by https://github.com/sigmaSd/Rhook/blob/master/src/scaffold.rs)
//! - closure input (which is the libc function input)
//! - closure output (which is the libc function output)
//! - The original function with the following name `original_$libcfn` this is useful in particular to avoid recursion
//! - Some varaibles to make coding easier: `transmute` `ManuallyDrop` `CString` and a static mut `COUNTER`
//!
//! - You can find the input/output of a function by looking it up here [libc](https://docs.rs/libc)
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
//!1- Look up its doc's here  [recv](https://docs.rs/libc/0.2.93/libc/fn.recv.html) to see what the
//!function's input/output is
//!
//!2- use this crate
//!```rust
//!use rhook::{RunHook, Hook};
//!
//!std::process::Command::new("speedtest").add_hook(Hook::recv(stringify!(||{
//!  std::thread::sleep(std::time::Duration::from_millis(10));
//!  // since we're not doing any modification to the output you can just return None here
//!  Some(original_recv(socket, buf, len, flags))
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
//!Check out the examples for more info

#[cfg(not(unix))]
compile_error!("This crate is unix only");

pub(crate) mod libcfn;
use std::{
    cell::RefCell,
    collections::HashSet,
    io::{self, Write},
    sync::Mutex,
};

use once_cell::sync::Lazy;
use std::io::Result;
use std::process::{Command, Stdio};

thread_local! {
static HOOKS: RefCell<HashSet<Hook>> = RefCell::new(HashSet::new());
}

static RHOOK_DYNLIB_DIR_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

mod hook;
pub use hook::Hook;

/// Specify libc hooks for a Command
pub trait RunHook {
    /// Add a libc hook to the command
    fn add_hook(&mut self, hook: Hook) -> &mut Self {
        HOOKS.with(|hooks| hooks.borrow_mut().insert(hook));
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
        //only one Command should do the next lines at a given time
        //take lock here
        let _lock = RHOOK_DYNLIB_DIR_LOCK.lock().expect("should not happen");

        prepare()?;

        let mut drained_hooks = HashSet::new();
        HOOKS.with(|hooks| drained_hooks = hooks.borrow_mut().drain().collect());
        for hook in drained_hooks {
            append(hook.function())?;
        }
        build_dylib()?;

        //drop lock here
        drop(_lock);

        Ok(self.env("LD_PRELOAD", "/tmp/rhookdyl/target/debug/librhookdyl.so"))
    }
}

/// Create the dynamic library and write the scaffold to it
fn prepare() -> Result<()> {
    const CARGO_TOML: &str = r#"[package]
name = "rhookdyl"
version = "0.1.0"
edition = "2018"
[lib]
crate-type = ["dylib"]
[dependencies]
libc = "0.2.92""#;

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
    let status = Command::new("cargo")
        .arg("b")
        .current_dir("/tmp/rhookdyl")
        .env("CARGO_TARGET_DIR", "/tmp/rhookdyl/target")
        .spawn()?
        .wait()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to compile the dynamic library",
        ))
    }
}
