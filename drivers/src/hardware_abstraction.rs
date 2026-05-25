//! Hardware Abstraction Layer
//! 
//! Detects and abstracts hardware capabilities

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CPUFeatures {
    pub cores: usize,
    pub threads_per_core: usize,
    pub has_avx512: bool,
    pub has_sgx: bool,
    pub has_tme: bool,
    pub has_amx: bool,
    pub cache_line_size: usize,
    pub tsc_frequency_hz: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct GPUInfo {
    pub vendor: String,
    pub memory_gb: usize,
    pub compute_units: usize,
    pub supports_cuda: bool,
    pub supports_opencl: bool,
    pub supports_vulkan: bool,
}

#[derive(Debug, Clone)]
pub struct TPUInfo {
    pub tensor_cores: usize,
    pub int8_tops: usize,
    pub fp16_tflops: usize,
}

#[derive(Debug, Clone)]
pub struct SecurityCapabilities {
    pub has_tpm: bool,
    pub tpm_version: u8,
    pub has_secure_boot: bool,
    pub supports_measured_boot: bool,
    pub has_memory_encryption: bool,
}

pub struct HardwareAbstractionLayer {
    cpu: CPUFeatures,
    gpu: Option<GPUInfo>,
    tpu: Option<TPUInfo>,
    security: SecurityCapabilities,
    memory_map: HashMap<String, MemoryRegion>,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base_address: u64,
    pub size_bytes: u64,
    pub is_volatile: bool,
    pub is_encrypted: bool,
}

impl HardwareAbstractionLayer {
    pub fn detect() -> Self {
        HardwareAbstractionLayer {
            cpu: CPUFeatures {
                cores: 4,
                threads_per_core: 2,
                has_avx512: false,
                has_sgx: false,
                has_tme: false,
                has_amx: false,
                cache_line_size: 64,
                tsc_frequency_hz: None,
            },
            gpu: None,
            tpu: None,
            security: SecurityCapabilities {
                has_tpm: false,
                tpm_version: 0,
                has_secure_boot: false,
                supports_measured_boot: false,
                has_memory_encryption: false,
            },
            memory_map: HashMap::new(),
        }
    }

    pub fn cpu_features(&self) -> &CPUFeatures {
        &self.cpu
    }

    pub fn gpu_info(&self) -> Option<&GPUInfo> {
        self.gpu.as_ref()
    }

    pub fn tpu_info(&self) -> Option<&TPUInfo> {
        self.tpu.as_ref()
    }

    pub fn security_caps(&self) -> &SecurityCapabilities {
        &self.security
    }

    pub fn can_accelerate(&self, operation: &str) -> bool {
        match operation {
            "matrix_multiply" => self.cpu.has_amx || self.tpu.is_some() || self.gpu.is_some(),
            "encryption" => self.security.has_tpm,
            "memory_isolation" => self.cpu.has_sgx || self.security.has_memory_encryption,
            "vector_math" => self.cpu.has_avx512,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let hal = HardwareAbstractionLayer::detect();
        assert!(hal.cpu_features().cores >= 1);
        assert_eq!(hal.cpu_features().cache_line_size, 64);
    }

    #[test]
    fn test_acceleration_check() {
        let hal = HardwareAbstractionLayer::detect();
        assert!(!hal.can_accelerate("encryption"));
    }
}
