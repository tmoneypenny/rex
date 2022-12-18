use libc;
use log::{debug, error};
use std::io::prelude::*;
use std::os::raw::{c_char, c_void};
use std::{ffi, process};

const ELF_NAME: &'static str = "REX"; // ELF FD name

fn main() -> Result<(), i8> {
    env_logger::init();
    debug!("Hello World");

    let fd_name = ffi::CString::new(ELF_NAME).expect("c_char couldn't be created");

    let mdf = unsafe {
        match libc::memfd_create(fd_name.as_ptr() as *const c_char, 0) {
            -1 => {
                error!("failed to created in-memory file descriptor");
                process::exit(1)
            }
            fd => {
                debug!("created in-memory file descriptor");
                fd
            }
        }
    };

    // read from stdin
    let mut buffer = Vec::new();
    let stdin = std::io::stdin();
    {
        let mut stdin_handle = stdin.lock();
        match stdin_handle.read_to_end(&mut buffer) {
            Ok(_) => (),
            Err(_) => {
                debug!("failed to read stdin to buffer");
                process::exit(1)
            }
        }
    }

    // writeall to mdf
    unsafe {
        let written = libc::write(
            mdf as i32,
            buffer.as_mut_ptr() as *const c_void,
            buffer.len() as usize,
        );
        debug!("Wrote {} bytes", written);
    };

    // TODO: fix argv + envp
    let args = [""];
    let env = [""];

    unsafe {
        match libc::fexecve(
            mdf as i32,
            args.as_ptr() as *const *const c_char,
            env.as_ptr() as *const *const c_char,
        ) {
            -1 => {
                error!("failed to execute in-memory file descriptor");
                process::exit(1)
            }
            _ => {
                debug!("executed in-memory file descriptor");
            }
        }
    }

    Ok(())
}
