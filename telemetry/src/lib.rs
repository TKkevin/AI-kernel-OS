//! Real-time Telemetry & Metrics Collection
//!
//! Complete system visibility with minimal overhead.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub cpu_utilization: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub io_read_ops: u64,
    pub io_write_ops: u64,
    pub context_switches: u64,
    pub page_faults: u64,
    pub interrupt_count: u64,
}

pub struct TelemetryCollector {
    metrics_buffer: Vec<SystemMetrics>,
    max_samples: usize,
}

impl TelemetryCollector {
    pub fn new(max_samples: usize) -> Self {
        TelemetryCollector {
            metrics_buffer: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Collect current system metrics
    pub fn collect(&mut self, metrics: SystemMetrics) {
        if self.metrics_buffer.len() >= self.max_samples {
            self.metrics_buffer.remove(0);
        }
        self.metrics_buffer.push(metrics);
    }

    /// Export metrics as JSON
    pub fn export_json(&self) -> String {
        serde_json::to_string_pretty(&self.metrics_buffer).unwrap_or_default()
    }

    /// Get current statistics
    pub fn get_stats(&self) -> TelemetryStats {
        if self.metrics_buffer.is_empty() {
            return TelemetryStats::default();
        }

        let avg_cpu: f32 = self.metrics_buffer.iter().map(|m| m.cpu_utilization).sum::<f32>()
            / self.metrics_buffer.len() as f32;

        TelemetryStats {
            sample_count: self.metrics_buffer.len(),
            avg_cpu_utilization: avg_cpu,
            peak_memory: self.metrics_buffer.iter().map(|m| m.memory_used).max().unwrap_or(0),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TelemetryStats {
    pub sample_count: usize,
    pub avg_cpu_utilization: f32,
    pub peak_memory: u64,
}
