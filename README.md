# Rhook

Hooks libc functions with an easy API

## Example

Say you want to limit the badnwhidth of a program

Usually downloading calls `libc::recv` function

So our goal is to throttle it with a simple sleep

To do that with this crate: (taking speedtest program as an example)

1- Check its manpage https://man7.org/linux/man-pages/man2/recv.2.html to see what is the
function's input/output

2- use this crate
```rust
use rhook::{run_with, Hook};

run_with(vec!("speedtest"), vec!(Hook::Recv(stringify!(|sockfd, buf, len, flags|{
  std::thread::sleep_ms(100);
  original_recv(sockfd, buf, len, flags)
}))))
```

Thats it!
Note that you have acess inside the closure to the original function denoted by the prefix
`original_` + the function name


Couple of points:
- If you take ownership of an input value inside of the closure, be sure to use ManuallyDrop so
you don't free it

- To check if a program dynamicly load libc use `ldd $path_to_program`

- To check what libc functions a program calls use `nm -D $path_to_program`

Check out the tests for more examples
