//! NEXUS Kernel Core
//! 
//! The foundational layer of the NEXUS operating system.
//! Coordinates between hardware abstraction and higher-level subsystems.

#![no_std]

pub mod boot;
pub mod hardware;
pub mod memory;
pub mod interrupts;

/// Initialize the NEXUS kernel
pub unsafe fn initialize() {
    boot::initialize();
    hardware::detect();
    memory::init();
    interrupts::setup();
}

/// Kernel version
pub const VERSION: &str = "0.1.0";

/// System name
pub const SYSTEM_NAME: &str = "NEXUS";

pub fn hello() {
    println!("═══════════════════════════════════════");
    println!("  {} Kernel v{}", SYSTEM_NAME, VERSION);
    println!("  AI-Driven Operating System");
    println!("  Building what the future needs");
    println!("═══════════════════════════════════════");
}
