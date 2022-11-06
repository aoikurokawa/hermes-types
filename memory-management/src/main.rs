use std::process::{id, Command};

const ALLOC_SIZE: i32 = 1024 * 1024 * 1024;

fn mmap() {
    let pid = id();
    println!("Memorymap before getting new memory space");

    let command = Command::new("/bin/cat")
        .arg(format!("/proc/{pid}/maps"))
        .output()
        .expect("Failed to create command");

    println!("{:?}", command);
}

fn main() {
    const SIZE: usize = 100000;

    mmap();

    //println!("Show memory usage in the system before getting memory\n");
    //let free_command = Command::new("free").output().expect("Failed to execute");
    //println!("{:?}", free_command);

    //let _array = [0; SIZE];
    //println!("Show memory usage in the system after getting memory\n");
    //let free_command = Command::new("free").output().expect("Failed to execute");
    //println!("{:?}", free_command);
}
