use std::process;

pub fn print_hello_world() {
    println!("My pid is {}", process::id());
}

pub fn syscall_inf_loop() {
    println!("My pid is {}", process::id());
}

pub fn main() {
    print_hello_world();
    // loop {
    //    syscall_inf_loop();
    //}
}
