//!Hook libc functions with an easy API
//!
//!## Usage
//!
//!1- Import the trait [RunHook]
//!
//!2- Create an [Command](std::process::Command) with [Command::new](std::process::Command::new) and add hooks to it via [add_hook](RunHook::add_hook) and [add_hooks](RunHook::add_hooks) methods
//!
//!3- Confirm the hooks with (Anchor::set_hooks) method this step is necessary
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
//! - You can find the input/output of a function by looking it up here [libc](https://docs.rs/libc)
//! - Add `.map_err(|e|println("{}",e))` after `set_hooks` in order to prettify the dynamic library compiling error while debugging
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
    collections::HashSet,
    io::{self, Write},
    sync::Mutex,
};

use io::Read;
use once_cell::sync::Lazy;
use std::io::Result;
use std::process::{Command, Stdio};

// synchronize dynamic library building between different threads
static RHOOK_DYNLIB_DIR_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

mod hook;
pub use hook::Hook;

/// The struct that holds the current command hooks
///
/// It is created by calling add_hook/add_hooks on Command
///
/// This struct guarantees by the type system that [Anchor::set_hooks] is called
pub struct Anchor<'a> {
    command: Option<&'a mut Command>,
    hooks: Option<HashSet<Hook>>,
}

impl<'a> Anchor<'a> {
    /// Set the hooks, this is a required method since it does the actual work of creating a
    /// dynamic library and linking the target program with it
    pub fn set_hooks(&mut self) -> Result<&mut Command> {
        //only one Command should do the next lines at a given time
        //take lock here
        let _lock = RHOOK_DYNLIB_DIR_LOCK.lock().expect("should not happen");

        prepare()?;

        for hook in self
            .hooks
            .as_ref()
            .expect(Self::FIELDS_ARE_ALWAYS_NOT_NONE_JUSTIFICATION)
        {
            append(hook.function())?;
        }
        build_dylib()?;

        //drop lock here
        drop(_lock);

        Ok(self
            .command
            .take()
            .expect(Self::FIELDS_ARE_ALWAYS_NOT_NONE_JUSTIFICATION)
            .env("LD_PRELOAD", "/tmp/rhookdyl/target/debug/librhookdyl.so"))
    }

    //-----------------
    // private methods
    //-----------------

    fn new(command: &'a mut Command) -> Self {
        Self {
            command: Some(command),
            hooks: Some(HashSet::new()),
        }
    }

    fn insert_hook(&mut self, hook: Hook) {
        self.hooks
            .as_mut()
            .expect(Self::FIELDS_ARE_ALWAYS_NOT_NONE_JUSTIFICATION)
            .insert(hook);
    }

    fn insert_hooks(&mut self, hooks: Vec<Hook>) {
        hooks.into_iter().for_each(|hook| self.insert_hook(hook));
    }

    const FIELDS_ARE_ALWAYS_NOT_NONE_JUSTIFICATION: &'a str = "Anchor can not be created outside of this library (its fields are private and the new method is also private), and we're guaranteeing internally that each time we construct an Anchor that its fields are set";
}

/// Specify libc hooks for a Command
pub trait RunHook {
    /// Add a libc hook to the command
    fn add_hook(&mut self, hook: Hook) -> Anchor;
    /// Add a Vec of libc hooks to the command
    fn add_hooks(&mut self, hooks: Vec<Hook>) -> Anchor;
}

impl RunHook for Anchor<'_> {
    fn add_hook(&mut self, hook: Hook) -> Anchor {
        self.insert_hook(hook);
        Self {
            command: self.command.take(),
            hooks: self.hooks.take(),
        }
    }

    fn add_hooks(&mut self, hooks: Vec<Hook>) -> Anchor {
        self.insert_hooks(hooks);
        Self {
            command: self.command.take(),
            hooks: self.hooks.take(),
        }
    }
}

impl RunHook for Command {
    fn add_hook(&mut self, hook: Hook) -> Anchor {
        let mut anchor = Anchor::new(self);
        anchor.insert_hook(hook);
        anchor
    }
    fn add_hooks(&mut self, hooks: Vec<Hook>) -> Anchor {
        let mut anchor = Anchor::new(self);
        anchor.insert_hooks(hooks);
        anchor
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
    let mut stderr = String::new();
    let mut process = Command::new("cargo")
        .arg("b")
        .args(&["--color", "always"])
        .current_dir("/tmp/rhookdyl")
        .stderr(Stdio::piped())
        .env("CARGO_TARGET_DIR", "/tmp/rhookdyl/target")
        .spawn()?;
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
