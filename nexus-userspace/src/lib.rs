//! NEXUS OS Userspace Runtime Library
//! 
//! Provides POSIX-compatible APIs with Stark-level intelligence

#![no_std]

extern crate alloc;

use alloc::string::String;
use core::ffi::{c_int, c_void, c_char, c_long, c_ulong};

// Re-export core types
pub use nexus_core::syscall::*;

/// File descriptor types
pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;

/// Open flags (from Unix)
pub const O_RDONLY: c_int = 0;
pub const O_WRONLY: c_int = 1;
pub const O_RDWR: c_int = 2;
pub const O_CREAT: c_int = 0o100;
pub const O_EXCL: c_int = 0o200;
pub const O_TRUNC: c_int = 0o1000;
pub const O_APPEND: c_int = 0o2000;

/// Seek positions
pub const SEEK_SET: c_int = 0;
pub const SEEK_CUR: c_int = 1;
pub const SEEK_END: c_int = 2;

/// Process status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_mtime: i64,
    pub st_ctime: i64,
}

/// Timeval for select/gettimeofday
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Timeval {
    pub tv_sec: c_long,
    pub tv_usec: c_long,
}

/// Resource limits (from Unix rlimit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Rlimit {
    pub rlim_cur: c_ulong,
    pub rlim_max: c_ulong,
}

/// Signal constants (POSIX)
pub const SIGHUP: c_int = 1;
pub const SIGINT: c_int = 2;
pub const SIGQUIT: c_int = 3;
pub const SIGILL: c_int = 4;
pub const SIGTRAP: c_int = 5;
pub const SIGABRT: c_int = 6;
pub const SIGBUS: c_int = 7;
pub const SIGFPE: c_int = 8;
pub const SIGKILL: c_int = 9;
pub const SIGUSR1: c_int = 10;
pub const SIGSEGV: c_int = 11;
pub const SIGUSR2: c_int = 12;
pub const SIGPIPE: c_int = 13;
pub const SIGALRM: c_int = 14;
pub const SIGTERM: c_int = 15;
pub const SIGCHLD: c_int = 17;
pub const SIGCONT: c_int = 18;
pub const SIGSTOP: c_int = 19;
pub const SIGTSTP: c_int = 20;

/// Signal handler type
pub type SigHandler = extern "C" fn(c_int);

pub const SIG_DFL: SigHandler = 0 as SigHandler;
pub const SIG_IGN: SigHandler = 1 as SigHandler;

// ============================================================
// System Call Wrappers - The Bridge Between Userspace and Kernel
// ============================================================

/// Write to file descriptor
#[inline]
pub fn write(fd: c_int, buf: &[u8]) -> Result<usize, c_int> {
    let ret = unsafe {
        syscall1(SYS_WRITE, fd as usize, buf.as_ptr() as usize, buf.len())
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as usize)
    }
}

/// Read from file descriptor
#[inline]
pub fn read(fd: c_int, buf: &mut [u8]) -> Result<usize, c_int> {
    let ret = unsafe {
        syscall1(SYS_READ, fd as usize, buf.as_mut_ptr() as usize, buf.len())
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as usize)
    }
}

/// Open a file
#[inline]
pub fn open(path: &str, flags: c_int, mode: u32) -> Result<c_int, c_int> {
    let ret = unsafe {
        syscall3(SYS_OPEN, path.as_ptr() as usize, flags as usize, mode as usize)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as c_int)
    }
}

/// Close a file descriptor
#[inline]
pub fn close(fd: c_int) -> Result<(), c_int> {
    let ret = unsafe { syscall1(SYS_CLOSE, fd as usize) };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

/// Fork process (Unix semantics)
#[inline]
pub fn fork() -> Result<c_int, c_int> {
    let ret = unsafe { syscall0(SYS_FORK) };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as c_int)
    }
}

/// Execute a new program
#[inline]
pub fn execve(path: &str, args: &[&str], env: &[&str]) -> Result<(), c_int> {
    let ret = unsafe {
        syscall3(SYS_EXECVE, path.as_ptr() as usize, args.as_ptr() as usize, env.as_ptr() as usize)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

/// Wait for child process
#[inline]
pub fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> Result<c_int, c_int> {
    let ret = unsafe {
        syscall3(SYS_WAITPID, pid as usize, status as usize, options as usize)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as c_int)
    }
}

/// Exit current process
#[inline]
pub fn exit(status: c_int) -> ! {
    unsafe {
        syscall1(SYS_EXIT, status as usize);
        loop {} // Should never return
    }
}

/// Get process ID
#[inline]
pub fn getpid() -> c_int {
    unsafe { syscall0(SYS_GETPID) as c_int }
}

/// Get parent process ID
#[inline]
pub fn getppid() -> c_int {
    unsafe { syscall0(SYS_GETPPID) as c_int }
}

/// Send signal to process
#[inline]
pub fn kill(pid: c_int, sig: c_int) -> Result<(), c_int> {
    let ret = unsafe { syscall2(SYS_KILL, pid as usize, sig as usize) };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

/// Set signal handler
#[inline]
pub fn signal(sig: c_int, handler: SigHandler) -> Result<SigHandler, c_int> {
    let ret = unsafe { syscall2(SYS_SIGNAL, sig as usize, handler as usize) };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as SigHandler)
    }
}

/// Get time of day
#[inline]
pub fn gettimeofday(tv: *mut Timeval, tz: *mut c_void) -> Result<c_int, c_int> {
    let ret = unsafe { syscall2(SYS_GETTIMEOFDAY, tv as usize, tz as usize) };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as c_int)
    }
}

/// Memory map file into memory
#[inline]
pub fn mmap(addr: *mut c_void, length: usize, prot: c_int, flags: c_int, fd: c_int, offset: i64) -> Result<*mut c_void, c_int> {
    let ret = unsafe {
        syscall6(SYS_MMAP, addr as usize, length, prot as usize, flags as usize, fd as usize, offset as usize)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as *mut c_void)
    }
}

/// Unmap memory region
#[inline]
pub fn munmap(addr: *mut c_void, length: usize) -> Result<(), c_int> {
    let ret = unsafe { syscall2(SYS_MUNMAP, addr as usize, length) };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

/// Brk system call for heap management
#[inline]
pub fn brk(addr: *mut c_void) -> Result<*mut c_void, c_int> {
    let ret = unsafe { syscall1(SYS_BRK, addr as usize) };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as *mut c_void)
    }
}

/// Get file status
#[inline]
pub fn stat(path: &str, buf: *mut Stat) -> Result<c_int, c_int> {
    let ret = unsafe {
        syscall2(SYS_STAT, path.as_ptr() as usize, buf as usize)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as c_int)
    }
}

/// Change working directory
#[inline]
pub fn chdir(path: &str) -> Result<(), c_int> {
    let ret = unsafe { syscall1(SYS_CHDIR, path.as_ptr() as usize) };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

/// Get current working directory
#[inline]
pub fn getcwd(buf: &mut [u8]) -> Result<&str, c_int> {
    let ret = unsafe {
        syscall2(SYS_GETCWD, buf.as_mut_ptr() as usize, buf.len())
    };
    if ret < 0 {
        Err(ret)
    } else {
        // Safe because kernel guarantees null-terminated string
        Ok(core::str::from_utf8(unsafe {
            core::slice::from_raw_parts(buf.as_ptr(), ret as usize)
        }).unwrap())
    }
}

// ============================================================
// Utility Functions
// ============================================================

/// Print to stdout
pub fn println(s: &str) {
    let _ = write(STDOUT_FILENO, s.as_bytes());
    let _ = write(STDOUT_FILENO, b"\n");
}

/// Print to stderr
pub fn eprintln(s: &str) {
    let _ = write(STDERR_FILENO, s.as_bytes());
    let _ = write(STDERR_FILENO, b"\n");
}

/// Simple string formatting
pub fn format_int(n: i64) -> [u8; 32] {
    let mut buf = [0u8; 32];
    let mut i = 31;
    let mut num = n;
    
    if num == 0 {
        buf[31] = b'0';
        return buf;
    }
    
    let negative = num < 0;
    if negative {
        num = -num;
    }
    
    while num > 0 && i > 0 {
        buf[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i -= 1;
    }
    
    if negative && i > 0 {
        buf[i] = b'-';
        i -= 1;
    }
    
    buf
}
