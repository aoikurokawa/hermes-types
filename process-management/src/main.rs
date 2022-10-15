use nix::sys::wait::wait;
use nix::unistd::ForkResult::{Child, Parent};
use nix::unistd::{execve, fork, getpid, getppid};
use std::ffi::{CStr, CString};

fn practice_fork() {
    let pid = fork();

    match pid.expect("Fork Failed: Unable to create child process!") {
        Child => println!(
            "Hello from child process with pid: {} and parent pid: {}",
            getpid(),
            getppid()
        ),
        Parent { child } => {
            wait().expect("Failed to wait");
            println!(
                "Hello from parent process with pid: {} and child pid: {}",
                getpid(),
                child
            );
        }
    }
}

fn fork_and_exec() {
    let pid = fork();

    match pid.expect("Fork Failed: Unable to create child process!") {
        Child => {
            println!(
                "Hello from child process with pid: {} and parent pid: {}",
                getpid(),
                getppid()
            );

            let path = CString::new("/bin/echo").expect("");
            let args1 = CString::new("echo").expect("");
            let args2 = CString::new(format!("Hello from pid {}", getpid())).expect("");
            execve(&path, &[&args1, &args2], &[]).unwrap();
        }
        Parent { child } => {
            wait().expect("Failed to wait");
            println!(
                "Hello from parent process with pid: {} and child pid: {}",
                getpid(),
                child
            );
        }
    }
}

fn main() {
    fork_and_exec();
}
