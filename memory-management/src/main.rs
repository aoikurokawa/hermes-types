use std::process::Command;

fn main() {
    const SIZE: usize = 100000;

    println!("Show memory usage in the system before getting memory\n");
    let free_command = Command::new("free").output().expect("Failed to execute");
    println!("{:?}", free_command);

    let _array = [0; SIZE];
    println!("Show memory usage in the system after getting memory\n");
    let free_command = Command::new("free").output().expect("Failed to execute");
    println!("{:?}", free_command);

}
