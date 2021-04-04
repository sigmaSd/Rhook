# Rhook

Hook libc functions with an easy API

### Usage

1- Import the trait [RunHook]

2- Create an [Command](std::process::Command) with [Command::new](std::process::Command::new) and add hooks to it via [add_hook](RunHook::add_hook) and [add_hooks](RunHook::add_hooks) methods

3- Confirm the hooks with [set_hooks](RunHook::set_hooks) method this step is necessary

4- Now you can carry on with the usual [Command](std::process::Command) methods ([output](std::process::Command::output), [spawn](std::process::Command::spawn),[status](std::process::Command::status),..)

### Example

Say you want to limit the bandwidth of a program

Usually downloading calls `libc::recv` function

So our goal is to throttle it with a simple sleep

To do that with this crate: (taking speedtest program as an example)

1- Check its manpage https://man7.org/linux/man-pages/man2/recv.2.html to see what is the
function's input/output

2- use this crate
```rust
use rhook::{RunHook, Hook};

std::process::Command::new("speedtest").add_hook(Hook::Recv(stringify!(|sockfd, buf, len, flags|{
 std::thread::sleep_ms(10);
 original_recv(sockfd, buf, len, flags)
}))).set_hooks().unwrap().spawn();
```

Thats it!
Note that you have acess inside the closure to the original function denoted by the prefix
`original_` + the function name

Couple of points:
- If you take ownership of an input value inside of the closure, be sure to use ManuallyDrop so
you don't free it

- To check if a program dynamicly loads libc use `ldd $path_to_program`

- To check what libc functions a program calls use `nm -D $path_to_program`

Check out the tests for more examples

License: MIT
