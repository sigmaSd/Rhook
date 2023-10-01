# rhook

Hook libc functions with an easy API

### Usage

1- Import the trait [RunHook]

2- Create an [Command](std::process::Command) with [Command::new](std::process::Command::new) and add hooks to it via [add_hook](RunHook::add_hook) and [add_hooks](RunHook::add_hooks) methods

3- Confirm the hooks with [set_hooks](Anchor::set_hooks) method this step is necessary

3.1- Hooks are closures that takes no input and return an option of the libc function as output.

If the closure return `None` that is equivalent to returning `Some(original_function(args))` in
other words it will run and use the original function output

Inside the closure you have access to the libc function input + some imports from std (see
src/scaffold.rs)

4- Now you can carry on with the usual [Command](std::process::Command) methods ([output](std::process::Command::output), [spawn](std::process::Command::spawn),[status](std::process::Command::status),..)


**Tricks:**

The closure used for hooks have acess to many things: (imported by https://github.com/sigmaSd/Rhook/blob/master/src/scaffold.rs)
- closure input (which is the libc function input)
- closure output (which is the libc function output)
- The original function with the following name `original_$libcfn` this is useful in particular to avoid recursion
- Some varaibles to make coding easier: `transmute` `ManuallyDrop` `CString` and a static mut `COUNTER`
- You can find the input/output of a function by looking it up here [libc](https://docs.rs/libc)
- Add `.map_err(|e|println!("{}",e))` after `set_hooks` in order to prettify the dynamic library compiling error while debugging
- If you take ownership of an input value inside of the closure, be sure to use ManuallyDrop so you don't free it

### Example

Say you want to limit the bandwidth of a program

Usually downloading calls `libc::recv` function

So our goal is to throttle it with a simple sleep

To do that with this crate: (taking speedtest program as an example)

1- Look up its doc's here  [recv](https://docs.rs/libc/0.2.93/libc/fn.recv.html) to see what the
function's input/output is

2- use this crate
```rust
use rhook::{Hook, RunHook};

fn main() {
    std::process::Command::new("speedtest")
        .add_hook(Hook::recv(stringify!(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            // since we're not doing any modification to the output you can just return None here
            Some(original_recv(socket, buf, len, flags))
        })))
        .set_hooks()
        //  uncomment this line to get readable error in case of a panic
        // .map_err(|e| println!("{}", e))
        .unwrap()
        .spawn()
        .unwrap();
}
```

Thats it!
Note that you have acess inside the closure to the original function denoted by the prefix
`original_` + the function name

Check out the examples for more info

License: MIT
