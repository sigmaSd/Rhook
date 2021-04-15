use rhook::*;
use std::process::Command;

// return fake data for read
fn fake_cat() {
    Command::new("cat")
        .arg("Cargo.toml")
        .add_hooks(vec![
            Hook::read(stringify!(|| { Some(0) })),
            Hook::open(stringify!(|| { None })),
        ])
        .add_hook(Hook::read(stringify!(|| { Some(4) })))
        .add_hooks(vec![Hook::read(stringify!(|| { Some(3) }))])
        .add_hook(Hook::read(stringify!(|| { Some(1) })))
        .set_hooks()
        .map_err(|e| println!("{}", e))
        .unwrap()
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn main() {
    fake_cat();
}
