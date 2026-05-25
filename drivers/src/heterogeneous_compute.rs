//! Heterogeneous Compute Scheduler

use crate::hardware_abstraction::HardwareAbstractionLayer;

pub enum ComputeDevice {
    CPU(usize),
    GPU(usize),
    TPU(usize),
}

pub struct WorkloadDescriptor {
    pub parallelism: usize,
    pub memory_requirement_mb: usize,
    pub prefers_matrix_ops: bool,
    pub latency_sensitive: bool,
    pub data_parallel: bool,
}

pub struct HeterogeneousScheduler {
    hal: HardwareAbstractionLayer,
    device_load: std::collections::HashMap<usize, f32>,
}

impl HeterogeneousScheduler {
    pub fn new(hal: HardwareAbstractionLayer) -> Self {
        HeterogeneousScheduler {
            hal,
            device_load: std::collections::HashMap::new(),
        }
    }

    pub fn schedule(&mut self, workload: &WorkloadDescriptor) -> ComputeDevice {
        if workload.prefers_matrix_ops {
            if self.hal.tpu_info().is_some() {
                return ComputeDevice::TPU(0);
            }
            if self.hal.gpu_info().is_some() {
                return ComputeDevice::GPU(0);
            }
        }

        if workload.data_parallel && workload.parallelism > 100 {
            if self.hal.gpu_info().is_some() {
                return ComputeDevice::GPU(0);
            }
        }

        if workload.latency_sensitive {
            return ComputeDevice::CPU(0);
        }

        ComputeDevice::CPU(0)
    }

    pub fn update_load(&mut self, device_id: usize, load_delta: f32) {
        let load = self.device_load.entry(device_id).or_insert(0.0);
        *load = (*load + load_delta).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heterogeneous_scheduling() {
        let hal = HardwareAbstractionLayer::detect();
        let mut scheduler = HeterogeneousScheduler::new(hal);
        
        let workload = WorkloadDescriptor {
            parallelism: 10,
            memory_requirement_mb: 100,
            prefers_matrix_ops: false,
            latency_sensitive: true,
            data_parallel: false,
        };
        
        let device = scheduler.schedule(&workload);
        assert!(matches!(device, ComputeDevice::CPU(_)));
    }
}
