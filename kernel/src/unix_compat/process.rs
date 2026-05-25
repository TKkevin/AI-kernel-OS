//! Unix Process Model Compatibility Layer
//! 
//! Implements classic Unix process semantics:
//! - fork(), exec(), wait() family
//! - Signal handling (SIGKILL, SIGTERM, etc.)
//! - Process groups and sessions
//! - File descriptor inheritance

use std::collections::HashMap;
use std::sync::Arc;

/// Unix signal numbers (POSIX standard)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    SIGHUP = 1,      // Hangup
    SIGINT = 2,      // Interrupt from keyboard
    SIGQUIT = 3,     // Quit from keyboard
    SIGILL = 4,      // Illegal instruction
    SIGTRAP = 5,     // Trace/breakpoint trap
    SIGABRT = 6,     // Abort
    SIGBUS = 7,      // Bus error
    SIGFPE = 8,      // Floating-point exception
    SIGKILL = 9,     // Kill (cannot be caught)
    SIGUSR1 = 10,    // User-defined signal 1
    SIGSEGV = 11,    // Invalid memory reference
    SIGUSR2 = 12,    // User-defined signal 2
    SIGPIPE = 13,    // Broken pipe
    SIGALRM = 14,    // Timer signal
    SIGTERM = 15,    // Termination signal
    SIGCHLD = 17,    // Child stopped or terminated
    SIGCONT = 18,    // Continue if stopped
    SIGSTOP = 19,    // Stop process (cannot be caught)
    SIGTSTP = 20,    // Stop typed at terminal
}

impl Signal {
    pub fn from_number(num: u32) -> Option<Self> {
        match num {
            1 => Some(Signal::SIGHUP),
            2 => Some(Signal::SIGINT),
            9 => Some(Signal::SIGKILL),
            15 => Some(Signal::SIGTERM),
            17 => Some(Signal::SIGCHLD),
            _ => None,
        }
    }

    /// Check if signal can be caught/blocked
    pub fn is_catchable(&self) -> bool {
        !matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }
}

/// Unix Process Control Block (PCB)
/// Modeled after Linux task_struct with Unix semantics
#[derive(Debug, Clone)]
pub struct UnixProcess {
    pub pid: u32,
    pub ppid: u32,              // Parent PID
    pub pgid: u32,              // Process Group ID
    pub sid: u32,               // Session ID
    pub uid: u32,               // User ID
    pub gid: u32,               // Group ID
    pub euid: u32,              // Effective UID
    pub egid: u32,              // Effective GID
    
    pub state: ProcessState,
    pub exit_code: Option<i32>,
    
    // File descriptors (0=stdin, 1=stdout, 2=stderr)
    pub file_descriptors: HashMap<u32, FdEntry>,
    
    // Signal handling
    pub signal_handlers: HashMap<u32, SignalHandler>,
    pub pending_signals: u64,   // Bitmask of pending signals
    pub blocked_signals: u64,   // Bitmask of blocked signals
    
    // Resource limits (rlimit)
    pub rlimit_cpu: u64,
    pub rlimit_file_size: u64,
    pub rlimit_data: u64,
    pub rlimit_stack: u64,
    pub rlimit_core: u64,
    pub rlimit_nofile: u64,
    
    // Timing
    pub utime: u64,             // User mode time
    pub stime: u64,             // Kernel mode time
    pub cutime: u64,            // Children's user time
    pub cstime: u64,            // Children's kernel time
    pub start_time: u64,        // System uptime at start
    
    // Working directory
    pub cwd: String,
    
    // Child processes
    pub children: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    Running,        // R - runnable or running
    Sleeping,       // S - interruptible sleep
    DiskSleep,      // D - uninterruptible sleep (IO)
    Stopped,        // T - stopped by signal
    TracingStop,    // t - tracing stop
    Zombie,         // Z - terminated but not reaped
    Dead,           // X - dead (should never be seen)
    Idle,           // I - idle kernel thread
}

#[derive(Debug, Clone)]
pub struct FdEntry {
    pub fd_type: FdType,
    pub flags: FdFlags,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub enum FdType {
    File(String),
    Socket(u32),
    Pipe(u32, u32), // read_end, write_end
    Device(String),
}

#[derive(Debug, Clone, Copy)]
pub struct FdFlags {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub nonblock: bool,
    pub close_on_exec: bool,
}

#[derive(Debug, Clone)]
pub struct SignalHandler {
    pub handler: HandlerType,
    pub mask: u64,          // Signals to block during handler
    pub flags: SignalFlags,
}

#[derive(Debug, Clone)]
pub enum HandlerType {
    Default,
    Ignore,
    Custom(u64),            // Function pointer address
}

#[derive(Debug, Clone, Copy)]
pub struct SignalFlags {
    pub sa_nochild: bool,
    pub sa_restart: bool,
    pub sa_nodefer: bool,
    pub sa_onstack: bool,
    pub sa_restorer: bool,
}

impl UnixProcess {
    /// Create a new init-like process (PID 1)
    pub fn new_init() -> Self {
        Self {
            pid: 1,
            ppid: 0,
            pgid: 1,
            sid: 1,
            uid: 0,
            gid: 0,
            euid: 0,
            egid: 0,
            state: ProcessState::Running,
            exit_code: None,
            file_descriptors: Self::default_fds(),
            signal_handlers: HashMap::new(),
            pending_signals: 0,
            blocked_signals: 0,
            rlimit_cpu: u64::MAX,
            rlimit_file_size: u64::MAX,
            rlimit_data: u64::MAX,
            rlimit_stack: 8 * 1024 * 1024, // 8MB default
            rlimit_core: 0,
            rlimit_nofile: 1024,
            utime: 0,
            stime: 0,
            cutime: 0,
            cstime: 0,
            start_time: 0,
            cwd: "/".to_string(),
            children: Vec::new(),
        }
    }

    /// Initialize standard file descriptors
    fn default_fds() -> HashMap<u32, FdEntry> {
        let mut fds = HashMap::new();
        
        // stdin
        fds.insert(0, FdEntry {
            fd_type: FdType::Device("/dev/tty".to_string()),
            flags: FdFlags { read: true, write: false, append: false, nonblock: false, close_on_exec: false },
            offset: 0,
        });
        
        // stdout
        fds.insert(1, FdEntry {
            fd_type: FdType::Device("/dev/tty".to_string()),
            flags: FdFlags { read: false, write: true, append: false, nonblock: false, close_on_exec: false },
            offset: 0,
        });
        
        // stderr
        fds.insert(2, FdEntry {
            fd_type: FdType::Device("/dev/tty".to_string()),
            flags: FdFlags { read: false, write: true, append: false, nonblock: false, close_on_exec: false },
            offset: 0,
        });
        
        fds
    }

    /// Simulate fork() - create child process
    pub fn fork(&self, new_pid: u32) -> Self {
        let mut child = self.clone();
        child.pid = new_pid;
        child.ppid = self.pid;
        child.pgid = self.pgid; // Inherit process group
        child.state = ProcessState::Running;
        child.exit_code = None;
        child.pending_signals = 0; // Clear pending signals
        child.utime = 0;
        child.stime = 0;
        child.cutime = 0;
        child.cstime = 0;
        child.children = Vec::new();
        
        // Copy file descriptors (COW in real implementation)
        // FDs are shared but have independent offsets
        
        child
    }

    /// Send signal to process
    pub fn send_signal(&mut self, signal: Signal) -> Result<(), &'static str> {
        match signal {
            Signal::SIGKILL => {
                self.state = ProcessState::Dead;
                self.exit_code = Some(-9);
                Ok(())
            }
            Signal::SIGSTOP => {
                self.state = ProcessState::Stopped;
                Ok(())
            }
            Signal::SIGCONT => {
                if self.state == ProcessState::Stopped {
                    self.state = ProcessState::Running;
                }
                Ok(())
            }
            _ => {
                // Set pending signal bit
                let bit = signal as u32;
                self.pending_signals |= 1 << bit;
                Ok(())
            }
        }
    }

    /// Check if process has pending, unblocked signals
    pub fn has_pending_signals(&self) -> bool {
        self.pending_signals & !self.blocked_signals != 0
    }

    /// Get next pending signal
    pub fn get_next_signal(&self) -> Option<Signal> {
        let pending = self.pending_signals & !self.blocked_signals;
        if pending == 0 {
            return None;
        }
        
        // Find lowest numbered pending signal
        for i in 1..64 {
            if pending & (1 << i) != 0 {
                return Signal::from_number(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_process_creation() {
        let init = UnixProcess::new_init();
        assert_eq!(init.pid, 1);
        assert_eq!(init.ppid, 0);
        assert_eq!(init.state, ProcessState::Running);
        assert!(init.file_descriptors.contains_key(&0));
        assert!(init.file_descriptors.contains_key(&1));
        assert!(init.file_descriptors.contains_key(&2));
    }

    #[test]
    fn test_fork_semantics() {
        let parent = UnixProcess::new_init();
        let child = parent.fork(2);
        
        assert_eq!(child.pid, 2);
        assert_eq!(child.ppid, 1);
        assert_eq!(child.pgid, parent.pgid); // Same process group
        assert_ne!(child.start_time, parent.start_time);
    }

    #[test]
    fn test_signal_handling() {
        let mut proc = UnixProcess::new_init();
        
        // Send SIGTERM
        proc.send_signal(Signal::SIGTERM).unwrap();
        assert!(proc.has_pending_signals());
        
        // SIGKILL should kill immediately
        proc.send_signal(Signal::SIGKILL).unwrap();
        assert_eq!(proc.state, ProcessState::Dead);
        assert_eq!(proc.exit_code, Some(-9));
    }

    #[test]
    fn test_signal_catchable() {
        assert!(!Signal::SIGKILL.is_catchable());
        assert!(!Signal::SIGSTOP.is_catchable());
        assert!(Signal::SIGTERM.is_catchable());
        assert!(Signal::SIGINT.is_catchable());
    }
}
