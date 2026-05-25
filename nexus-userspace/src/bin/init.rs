//! Init process - The first userspace process (PID 1)
//! 
//! Stark Philosophy: "Every system needs a guardian. This is ours."

#![no_std]
#![no_main]

extern crate alloc;
use nexus_userspace::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize standard file descriptors
    let _ = write(STDOUT_FILENO, b"NEXUS OS Init Process Starting...\n");
    
    // Create essential directories (in real impl)
    // mkdir("/dev", 0o755);
    // mkdir("/proc", 0o555);
    // mkdir("/sys", 0o555);
    // mkdir("/tmp", 0o1777);
    
    // Mount filesystems (in real impl)
    // mount("devfs", "/dev", "devfs", 0, null);
    // mount("procfs", "/proc", "procfs", 0, null);
    
    // Start system services
    let _ = write(STDOUT_FILENO, b"Starting system services...\n");
    
    // Fork shell process
    match fork() {
        Ok(0) => {
            // Child process - exec shell
            let _ = write(STDOUT_FILENO, b"Launching shell...\n");
            // In real implementation: execve("/bin/shell", &args, &env);
            
            // For now, just exit
            exit(0);
        }
        Ok(pid) => {
            let msg = b"Shell started with PID: ";
            let _ = write(STDOUT_FILENO, msg);
            let pid_str = format_int(pid as i64);
            let _ = write(STDOUT_FILENO, &pid_str[20..]);
            let _ = write(STDOUT_FILENO, b"\n");
        }
        Err(e) => {
            let _ = write(STDERR_FILENO, b"Failed to fork shell! Error: ");
            let err_str = format_int(e as i64);
            let _ = write(STDERR_FILENO, &err_str[20..]);
            let _ = write(STDERR_FILENO, b"\n");
        }
    }
    
    // Init main loop - reap zombie processes
    let _ = write(STDOUT_FILENO, b"Init entering main loop...\n");
    
    loop {
        // Wait for any child process
        let mut status: i32 = 0;
        match waitpid(-1, &mut status, 0) {
            Ok(pid) if pid > 0 => {
                // Child exited - could restart critical services
                let _ = write(STDOUT_FILENO, b"Child process exited\n");
            }
            Ok(_) => {
                // No children - sleep briefly
                // In real impl: nanosleep()
            }
            Err(_) => {
                // Error - continue anyway
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let _ = write(STDERR_FILENO, b"INIT PANIC!\n");
    loop {
        x86_64::instructions::hlt();
    }
}
