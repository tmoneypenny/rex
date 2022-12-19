use log::{debug, error};
use std::io::prelude::*;
use std::os::raw::{c_char, c_void};
use std::{ffi, process};

const ELF_NAME: &str = "REX"; // ELF FD name

fn main() -> Result<(), i8> {
    env_logger::init();

    let fd_name = ffi::CString::new(ELF_NAME).expect("c_char couldn't be created");

    // Create in-memory file descriptor
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

    // Read ELF from stdin
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

    // Write-all buf to memfd
    unsafe {
        let written = libc::write(
            mdf as i32,
            buffer.as_mut_ptr() as *const c_void,
            buffer.len() as usize,
        );
        debug!("Wrote {} bytes", written);
    };

    // Parse argv and envp
    let args = std::env::args()
        .map(|arg| ffi::CString::new(arg).unwrap())
        .collect::<Vec<ffi::CString>>();

    let mut c_args = args
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const c_char>>();
    c_args.push(std::ptr::null());

    let envp = std::env::vars()
        .map(|(k, v)| ffi::CString::new(format!("{}={}", k, v)).unwrap())
        .collect::<Vec<ffi::CString>>();
    let mut c_envp = envp
        .iter()
        .map(|envp| envp.as_ptr())
        .collect::<Vec<*const c_char>>();
    c_envp.push(std::ptr::null());

    // Execute in-memory file descriptor
    unsafe {
        match libc::fexecve(mdf as i32, c_args.as_ptr(), c_envp.as_ptr()) {
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
