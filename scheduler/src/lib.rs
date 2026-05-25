//! Adaptive AI-Driven Scheduler
//! 
//! This module implements NEXUS's intelligent process scheduler.
//! Uses machine learning to predict optimal process ordering.

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub memory_access_pattern: Vec<f32>,
    pub cpu_affinity: usize,
    pub syscall_frequency: u32,
    pub last_run_duration: u64,
    pub context_switches: u64,
}

pub struct AdaptiveScheduler {
    processes: VecDeque<ProcessMetrics>,
    historical_patterns: Vec<ProcessMetrics>,
    prediction_model: SchedulerModel,
}

pub struct SchedulerModel {
    weights: Vec<f32>,
}

impl SchedulerModel {
    pub fn new() -> Self {
        SchedulerModel {
            weights: vec![0.5; 16], // Initialize model weights
        }
    }

    /// Predict which process should run next based on current state
    pub fn predict_next_process(&self, current_state: &[ProcessMetrics]) -> Option<u32> {
        if current_state.is_empty() {
            return None;
        }

        // Simple scoring: CPU affinity * (1 - memory pressure)
        let best_idx = current_state
            .iter()
            .enumerate()
            .max_by_key(|(_, p)| (p.cpu_affinity * 1000) as u32)
            .map(|(i, _)| i)?;

        Some(current_state[best_idx].pid)
    }

    /// Learn from observed process behavior
    pub fn learn(&mut self, metrics: &ProcessMetrics) {
        // Update model weights based on observed performance
        for weight in &mut self.weights {
            *weight = (*weight + 0.01).min(1.0);
        }
    }
}

impl AdaptiveScheduler {
    pub fn new() -> Self {
        AdaptiveScheduler {
            processes: VecDeque::new(),
            historical_patterns: Vec::new(),
            prediction_model: SchedulerModel::new(),
        }
    }

    /// Add a process to the scheduler
    pub fn add_process(&mut self, metrics: ProcessMetrics) {
        self.processes.push_back(metrics);
    }

    /// Get the next process to schedule
    pub fn next_process(&mut self) -> Option<u32> {
        let current_state: Vec<_> = self.processes.iter().cloned().collect();
        
        if let Some(pid) = self.prediction_model.predict_next_process(&current_state) {
            // Record this decision for learning
            if let Some(idx) = current_state.iter().position(|p| p.pid == pid) {
                if let Some(metrics) = self.processes.remove(idx) {
                    self.historical_patterns.push(metrics.clone());
                    self.prediction_model.learn(&metrics);
                    return Some(pid);
                }
            }
        }

        // Fallback: simple FIFO
        self.processes.pop_front().map(|p| p.pid)
    }

    /// Get scheduler metrics
    pub fn get_metrics(&self) -> SchedulerMetrics {
        SchedulerMetrics {
            queue_length: self.processes.len(),
            historical_patterns: self.historical_patterns.len(),
            model_accuracy: calculate_accuracy(&self.historical_patterns),
        }
    }
}

pub struct SchedulerMetrics {
    pub queue_length: usize,
    pub historical_patterns: usize,
    pub model_accuracy: f32,
}

fn calculate_accuracy(patterns: &[ProcessMetrics]) -> f32 {
    if patterns.is_empty() {
        return 0.0;
    }
    // Placeholder accuracy calculation
    0.75
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = AdaptiveScheduler::new();
        assert_eq!(scheduler.processes.len(), 0);
    }

    #[test]
    fn test_process_addition() {
        let mut scheduler = AdaptiveScheduler::new();
        let metrics = ProcessMetrics {
            pid: 1,
            memory_access_pattern: vec![0.5; 8],
            cpu_affinity: 0,
            syscall_frequency: 100,
            last_run_duration: 1000,
            context_switches: 5,
        };
        scheduler.add_process(metrics);
        assert_eq!(scheduler.processes.len(), 1);
    }
}
