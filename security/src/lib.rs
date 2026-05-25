//! Security Subsystem - Fortress Engine
//!
//! Real-time anomaly detection, behavioral analysis,
//! and automated threat response.

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct SystemBehaviorMetric {
    pub timestamp: u64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub io_operations: u32,
    pub syscall_rate: u32,
    pub context_switches: u32,
}

pub enum ThreatLevel {
    Safe,
    Suspicious,
    Critical,
}

pub struct AnomalyDetectionEngine {
    baseline_metrics: VecDeque<SystemBehaviorMetric>,
    threat_history: Vec<DetectedThreat>,
    sensitivity: f32,
}

pub struct DetectedThreat {
    pub timestamp: u64,
    pub threat_type: String,
    pub severity: f32,
    pub affected_process: u32,
}

impl AnomalyDetectionEngine {
    pub fn new(window_size: usize) -> Self {
        AnomalyDetectionEngine {
            baseline_metrics: VecDeque::with_capacity(window_size),
            threat_history: Vec::new(),
            sensitivity: 0.8,
        }
    }

    /// Analyze current system behavior for anomalies
    pub fn analyze(&mut self, metric: SystemBehaviorMetric) -> ThreatLevel {
        self.baseline_metrics.push_back(metric.clone());

        // Calculate deviation from baseline
        if let Some(deviation) = self.calculate_deviation(&metric) {
            if deviation > self.sensitivity {
                let threat = DetectedThreat {
                    timestamp: metric.timestamp,
                    threat_type: "Anomalous Behavior".to_string(),
                    severity: deviation,
                    affected_process: 0,
                };
                self.threat_history.push(threat);
                return ThreatLevel::Suspicious;
            }
        }

        ThreatLevel::Safe
    }

    fn calculate_deviation(&self, metric: &SystemBehaviorMetric) -> Option<f32> {
        if self.baseline_metrics.len() < 2 {
            return None;
        }

        // Calculate standard deviation from baseline
        let avg_cpu: f32 = self.baseline_metrics.iter().map(|m| m.cpu_usage).sum::<f32>()
            / self.baseline_metrics.len() as f32;
        
        let deviation = (metric.cpu_usage - avg_cpu).abs() / avg_cpu.max(0.01);
        Some(deviation.min(1.0))
    }

    /// Respond to detected threat
    pub fn respond_to_threat(&self, threat: &DetectedThreat) {
        match threat.severity {
            s if s > 0.9 => {
                // Critical: Isolate process
                println!("[SECURITY] Isolating suspicious process: {}", threat.affected_process);
            }
            s if s > 0.7 => {
                // High: Monitor closely
                println!("[SECURITY] Monitoring process: {}", threat.affected_process);
            }
            _ => {
                // Log for analysis
                println!("[SECURITY] Anomaly detected: {}", threat.threat_type);
            }
        }
    }
}
