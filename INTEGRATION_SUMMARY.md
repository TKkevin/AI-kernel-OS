# NEXUS OS - Integration of Unix, Linux, and ReactOS Concepts

## Overview

This document summarizes the integration of proven concepts from Unix, Linux, and ReactOS into the NEXUS kernel architecture.

---

## 1. Linux CFS Scheduler Integration

**Location:** `/workspace/scheduler/src/lib.rs`

### Concepts Adopted from Linux:

- **Virtual Runtime (vruntime)**: Core fairness metric that tracks how long a task has run, scaled by priority
- **Red-Black Tree Ordering**: Tasks sorted by vruntime for O(log n) scheduling decisions
- **Nice-to-Weight Conversion**: Direct implementation of Linux's `sched_prio_to_weight` table
- **Target Latency**: 6ms scheduling window to prevent starvation
- **Per-CPU Runqueues**: SMP-aware scheduling with CPU-local queues

### Key Implementation Details:

```rust
// Weight lookup table from Linux kernel (kernel/sched_fair.c)
const NICE_TO_WEIGHT: [u64; 40] = [
    88761, 71755, 56483, 46273, 36291,
    29154, 23254, 18705, 14949, 12307,
    10000, 8202,  6550,  5420,  4372,
    // ... continues for all 40 nice values
];

// Core CFS formula: vruntime += delta_exec * (NICE_0_LOAD / weight)
let weighted_delta = (delta_exec * NICE_0_LOAD) / task.weight.max(1);
```

### Stark Enhancement:
AI-driven boost factors adjust weights based on predicted I/O patterns and historical behavior.

---

## 2. Unix Process Model Integration

**Location:** `/workspace/kernel/src/unix_compat/process.rs`

### Concepts Adopted from Unix:

- **Process States**: Running, Sleeping, DiskSleep, Stopped, Traced, Zombie, Dead, Idle
- **Signal Handling**: Full POSIX signal numbers (SIGHUP, SIGINT, SIGKILL, SIGTERM, etc.)
- **File Descriptors**: Standard 0/1/2 (stdin/stdout/stderr) with inheritance
- **Process Groups & Sessions**: PGID and SID for job control
- **Resource Limits**: rlimit for CPU, file size, stack, core, nofile
- **fork() Semantics**: Copy-on-write process duplication model
- **UID/GID/EUID/EGID**: User identity and privilege separation

### Key Implementation Details:

```rust
#[repr(u32)]
pub enum Signal {
    SIGHUP = 1,      // Hangup
    SIGINT = 2,      // Interrupt from keyboard
    SIGKILL = 9,     // Kill (cannot be caught)
    SIGTERM = 15,    // Termination signal
    SIGCHLD = 17,    // Child stopped or terminated
}

pub struct UnixProcess {
    pub pid: u32,
    pub ppid: u32,
    pub pgid: u32,
    pub sid: u32,
    pub uid: u32,
    pub gid: u32,
    pub euid: u32,
    pub egid: u32,
    // ... full PCB structure
}
```

---

## 3. ReactOS Driver Architecture Integration

**Location:** `/workspace/drivers/src/reactos_style/driver_model.rs`

### Concepts Adopted from ReactOS/Windows NT:

- **IRP (I/O Request Packet)**: Unified request structure for all I/O operations
- **Major Function Codes**: 28 standardized operation types (Create, Read, Write, Close, etc.)
- **Driver Objects**: Container for driver state and dispatch tables
- **Device Objects**: Representation of hardware/logical devices
- **Dispatch Tables**: Function pointers for each major function
- **Device Stack**: Layered driver architecture with attached devices
- **Reference Counting**: Automatic lifetime management
- **NT Status Codes**: Standardized error/success returns

### Key Implementation Details:

```rust
pub struct Irp {
    pub major_function: MajorFunction,
    pub minor_function: u32,
    pub io_status: IoStatusBlock,
    pub user_buffer: Vec<u8>,
    pub parameters: IrpParameters,
}

pub enum MajorFunction {
    Create = 0,
    Close = 2,
    Read = 3,
    Write = 4,
    DeviceIoControl = 14,
    Pnp = 27,
    // ... 28 total functions
}

pub struct DriverObject {
    pub major_functions: [Option<DriverDispatch>; 28],
    pub device_objects: Vec<Arc<DeviceObject>>,
}
```

### Benefits:
- Clean separation between drivers and kernel
- Plug-and-play support built into architecture
- Filter drivers can layer without modifying base drivers
- Consistent error handling across all drivers

---

## 4. Additional Integrations

### Hardware Abstraction (`/workspace/drivers/src/hardware_abstraction.rs`)
- CPU feature detection (AVX-512, SGX, AMX)
- GPU/TPU heterogeneous compute support
- Security capabilities (TPM, Secure Boot, Memory Encryption)

### Heterogeneous Compute (`/workspace/drivers/src/heterogeneous_compute.rs`)
- Workload classification (matrix ops, latency-sensitive, data-parallel)
- Intelligent device selection (CPU/GPU/TPU)
- Load balancing across accelerators

### Performance Counters (`/workspace/drivers/src/performance_counters.rs`)
- CPU cycles, instructions retired, cache misses
- IPC (Instructions Per Cycle) calculation
- Cache miss rate monitoring
- Branch misprediction tracking

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     NEXUS Applications                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Unix Compatibility Layer                    │
│  • fork/exec/wait  • Signals  • File Descriptors  • POSIX   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Hybrid CFS Scheduler                       │
│  • Linux vruntime  • Nice weights  • AI boost  • SMP-aware  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│               ReactOS Driver Framework                       │
│  • IRPs  • Driver Objects  • Device Stack  • Dispatch Tables│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Hardware Abstraction Layer                      │
│  • CPU Features  • GPU/TPU  • Security  • Perf Counters     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        Physical Hardware                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Testing

All integrated components include comprehensive unit tests:

```bash
# Test scheduler (Linux CFS)
cargo test -p nexus-scheduler

# Test Unix compatibility
cargo test -p nexus-kernel -- unix_compat

# Test ReactOS driver model
cargo test -p nexus-drivers -- reactos_style
```

---

## Philosophy

> "We don't reinvent the wheel. We study the best wheels ever built, understand why they work, and build something that carries those ideas forward."

The NEXUS kernel integrates:
- **Unix's** elegant process model and signals
- **Linux's** fair scheduling and scalability
- **ReactOS's** clean driver architecture

Then enhances them with:
- **AI-driven** optimization and prediction
- **Modern Rust** safety guarantees
- **Heterogeneous compute** awareness

This is what Tony Stark would build: respecting proven engineering while pushing boundaries.
