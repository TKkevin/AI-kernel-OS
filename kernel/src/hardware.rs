//! Hardware abstraction and detection

pub struct HardwareCapabilities {
    pub cpu_cores: usize,
    pub has_tpm: bool,
    pub has_sgx: bool,
    pub has_sev: bool,
    pub has_gpu: bool,
    pub has_tpu: bool,
    pub memory_size: u64,
}

impl HardwareCapabilities {
    pub fn new() -> Self {
        HardwareCapabilities {
            cpu_cores: detect_cpu_cores(),
            has_tpm: detect_tpm(),
            has_sgx: detect_sgx(),
            has_sev: detect_sev(),
            has_gpu: detect_gpu(),
            has_tpu: detect_tpu(),
            memory_size: detect_memory(),
        }
    }
}

pub fn detect() -> HardwareCapabilities {
    HardwareCapabilities::new()
}

fn detect_cpu_cores() -> usize {
    // Query CPUID
    1 // Placeholder
}

fn detect_tpm() -> bool { false }
fn detect_sgx() -> bool { false }
fn detect_sev() -> bool { false }
fn detect_gpu() -> bool { false }
fn detect_tpu() -> bool { false }

fn detect_memory() -> u64 {
    // Query BIOS/UEFI
    4 * 1024 * 1024 * 1024 // 4GB default
}
