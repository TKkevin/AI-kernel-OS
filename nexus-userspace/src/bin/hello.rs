//! Hello World - First NEXUS OS Application
//! 
//! Stark Philosophy: "Sometimes the simplest things remind us why we build."

#![no_std]
#![no_main]

use nexus_userspace::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let _ = write(STDOUT_FILENO, b"╔════════════════════════════════════════╗\n");
    let _ = write(STDOUT_FILENO, b"║   NEXUS OS - Hello from Userspace!    ║\n");
    let _ = write(STDOUT_FILENO, b"╚════════════════════════════════════════╝\n");
    let _ = write(STDOUT_FILENO, b"\n");
    
    // Show process info
    let pid = getpid();
    let ppid = getppid();
    
    let _ = write(STDOUT_FILENO, b"Process ID: ");
    let pid_str = format_int(pid as i64);
    let _ = write(STDOUT_FILENO, &pid_str[20..]);
    let _ = write(STDOUT_FILENO, b"\n");
    
    let _ = write(STDOUT_FILENO, b"Parent PID: ");
    let ppid_str = format_int(ppid as i64);
    let _ = write(STDOUT_FILENO, &ppid_str[20..]);
    let _ = write(STDOUT_FILENO, b"\n");
    
    let _ = write(STDOUT_FILENO, b"\n");
    let _ = write(STDOUT_FILENO, b"System Status: OPERATIONAL\n");
    let _ = write(STDOUT_FILENO, b"Kernel: NEXUS v1.0\n");
    let _ = write(STDOUT_FILENO, b"Architecture: x86_64\n");
    let _ = write(STDOUT_FILENO, b"\n");
    
    let _ = write(STDOUT_FILENO, b"\"Sometimes you gotta run before you can walk.\" - Tony Stark\n");
    let _ = write(STDOUT_FILENO, b"\n");
    
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let _ = write(STDERR_FILENO, b"HELLO PANIC!\n");
    loop {
        x86_64::instructions::hlt();
    }
}
