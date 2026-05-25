//! NEXUS Shell - Command-line interface
//! 
//! Stark Philosophy: "The best interface is the one that understands what you mean, not just what you say."

#![no_std]
#![no_main]

extern crate alloc;
use alloc::{string::String, vec::Vec};
use nexus_userspace::*;

const PROMPT: &[u8] = b"nexus> ";
const MAX_LINE: usize = 1024;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let _ = write(STDOUT_FILENO, b"NEXUS Shell v1.0\n");
    let _ = write(STDOUT_FILENO, b"Type 'help' for commands.\n\n");
    
    let mut line_buf = [0u8; MAX_LINE];
    
    loop {
        // Print prompt
        let _ = write(STDOUT_FILENO, PROMPT);
        
        // Read line
        match read_line(&mut line_buf) {
            Ok(0) => {
                // EOF - exit shell
                break;
            }
            Ok(len) => {
                // Process command
                process_command(&line_buf[..len]);
            }
            Err(_) => {
                let _ = write(STDERR_FILENO, b"Read error\n");
            }
        }
    }
    
    let _ = write(STDOUT_FILENO, b"Goodbye.\n");
    exit(0);
}

fn read_line(buf: &mut [u8]) -> Result<usize, c_int> {
    let mut i = 0;
    while i < buf.len() - 1 {
        let mut byte = 0u8;
        match read(STDIN_FILENO, core::slice::from_mut(&mut byte)) {
            Ok(0) => return Ok(i), // EOF
            Ok(_) => {
                if byte == b'\n' || byte == b'\r' {
                    buf[i] = b'\n';
                    return Ok(i + 1);
                }
                if byte == b'\x7f' || byte == b'\x08' {
                    // Backspace
                    if i > 0 {
                        i -= 1;
                        let _ = write(STDOUT_FILENO, b"\x08 \x08");
                    }
                    continue;
                }
                buf[i] = byte;
                let _ = write(STDOUT_FILENO, &[byte]);
                i += 1;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(i)
}

fn process_command(line: &[u8]) {
    // Trim whitespace
    let trimmed = trim_whitespace(line);
    if trimmed.is_empty() {
        return;
    }
    
    // Parse command and arguments
    let mut parts = trimmed.split(|&b| b == b' ');
    let cmd = parts.next().unwrap();
    let args: Vec<&[u8]> = parts.collect();
    
    match cmd {
        b"help" => cmd_help(),
        b"echo" => cmd_echo(&args),
        b"pwd" => cmd_pwd(),
        b"ls" => cmd_ls(&args),
        b"cd" => cmd_cd(&args),
        b"ps" => cmd_ps(),
        b"exit" => { /* handled in main loop */ },
        b"clear" => cmd_clear(),
        b"whoami" => cmd_whoami(),
        b"date" => cmd_date(),
        b"cat" => cmd_cat(&args),
        _ => {
            let _ = write(STDERR_FILENO, b"Unknown command: ");
            let _ = write(STDERR_FILENO, cmd);
            let _ = write(STDERR_FILENO, b"\n");
        }
    }
}

fn cmd_help() {
    let _ = write(STDOUT_FILENO, b"Available commands:\n");
    let _ = write(STDOUT_FILENO, b"  help     - Show this help\n");
    let _ = write(STDOUT_FILENO, b"  echo     - Print text\n");
    let _ = write(STDOUT_FILENO, b"  pwd      - Print working directory\n");
    let _ = write(STDOUT_FILENO, b"  ls       - List directory\n");
    let _ = write(STDOUT_FILENO, b"  cd       - Change directory\n");
    let _ = write(STDOUT_FILENO, b"  ps       - List processes\n");
    let _ = write(STDOUT_FILENO, b"  clear    - Clear screen\n");
    let _ = write(STDOUT_FILENO, b"  whoami   - Show current user\n");
    let _ = write(STDOUT_FILENO, b"  date     - Show current time\n");
    let _ = write(STDOUT_FILENO, b"  cat      - Display file contents\n");
    let _ = write(STDOUT_FILENO, b"  exit     - Exit shell\n");
}

fn cmd_echo(args: &[&[u8]]) {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            let _ = write(STDOUT_FILENO, b" ");
        }
        let _ = write(STDOUT_FILENO, arg);
    }
    let _ = write(STDOUT_FILENO, b"\n");
}

fn cmd_pwd() {
    let mut buf = [0u8; 512];
    match getcwd(&mut buf) {
        Ok(path) => {
            let _ = write(STDOUT_FILENO, path.as_bytes());
            let _ = write(STDOUT_FILENO, b"\n");
        }
        Err(_) => {
            let _ = write(STDOUT_FILENO, b"/\n");
        }
    }
}

fn cmd_ls(_args: &[&[u8]]) {
    // Simplified - would list directory in real impl
    let _ = write(STDOUT_FILENO, b"bin  dev  etc  home  lib  proc  sys  tmp  usr  var\n");
}

fn cmd_cd(args: &[&[u8]]) {
    if args.is_empty() {
        let _ = chdir("/home");
    } else {
        // Convert &[u8] to &str for chdir
        // In real impl, would handle this properly
        let _ = write(STDOUT_FILENO, b"cd not fully implemented\n");
    }
}

fn cmd_ps() {
    // Simplified process listing
    let _ = write(STDOUT_FILENO, b"  PID  PPID  COMMAND\n");
    let _ = write(STDOUT_FILENO, b"    1     0  init\n");
    let _ = write(STDOUT_FILENO, b"    2     1  shell\n");
}

fn cmd_clear() {
    // ANSI escape codes to clear screen
    let _ = write(STDOUT_FILENO, b"\x1b[2J\x1b[H");
}

fn cmd_whoami() {
    let _ = write(STDOUT_FILENO, b"root\n");
}

fn cmd_date() {
    let mut tv = Timeval { tv_sec: 0, tv_usec: 0 };
    match gettimeofday(&mut tv, core::ptr::null_mut()) {
        Ok(_) => {
            let secs = tv.tv_sec;
            let _ = write(STDOUT_FILENO, b"Timestamp: ");
            let sec_str = format_int(secs);
            let _ = write(STDOUT_FILENO, &sec_str[15..]);
            let _ = write(STDOUT_FILENO, b"\n");
        }
        Err(_) => {
            let _ = write(STDOUT_FILENO, b"Time unavailable\n");
        }
    }
}

fn cmd_cat(args: &[&[u8]]) {
    if args.is_empty() {
        let _ = write(STDERR_FILENO, b"Usage: cat <file>\n");
        return;
    }
    
    // In real impl, would open and read file
    let _ = write(STDOUT_FILENO, b"File reading not yet implemented\n");
}

fn trim_whitespace(s: &[u8]) -> &[u8] {
    let start = s.iter().position(|&c| c != b' ' && c != b'\t').unwrap_or(s.len());
    let end = s.iter().rposition(|&c| c != b' ' && c != b'\t' && c != b'\n' && c != b'\r')
        .map(|i| i + 1)
        .unwrap_or(start);
    &s[start..end]
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let _ = write(STDERR_FILENO, b"SHELL PANIC!\n");
    loop {
        x86_64::instructions::hlt();
    }
}
