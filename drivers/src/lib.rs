//! NEXUS Hardware Drivers
//! 
//! High-performance drivers that leverage hardware capabilities
//! for maximum performance and security.
//! 
//! Integrates:
//! - ReactOS IRP-based Driver Model
//! - Heterogeneous Compute Scheduling
//! - Performance Counter Management
//! 
//! "Use the best tool for the job. Even if you have to build it yourself." - Stark Philosophy

pub mod hardware_abstraction;
pub mod heterogeneous_compute;
pub mod performance_counters;
pub mod reactos_style;

// Re-export main modules for convenience
pub use hardware_abstraction::*;
pub use heterogeneous_compute::*;
pub use performance_counters::*;
pub use reactos_style::*;
