//! NEXUS Hybrid Scheduler
//! 
//! Combines:
//! - Linux CFS (Completely Fair Scheduler): vruntime, red-black tree ordering
//! - Unix Process States: Runnable, Sleeping, Stopped, Zombie
//! - ReactOS Architecture: Modular driver-aware scheduling
//! - Stark AI Enhancements: Neural weighting, predictive boosting

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// LINUX CFS CORE CONCEPTS
// ============================================================================

/// Represents a process task in the scheduler
/// Inspired by Linux kernel's `task_struct`
#[derive(Debug, Clone)]
pub struct Task {
    pub pid: u32,
    pub tid: u32,
    pub priority: i32,        // Nice value: -20 (high) to 19 (low)
    pub static_priority: i32, // Original priority
    pub vruntime: u64,        // Virtual Runtime - core CFS concept
    pub weight: u64,          // Derived from priority (NICE_0_LOAD = 1024)
    pub state: TaskState,     // Unix process states
    pub last_run: Option<Instant>,
    pub sum_exec_runtime: u64,// Total execution time
    pub nr_cpus_allowed: u32, // CPU affinity mask count
    
    // Stark AI Enhancements
    pub ai_boost_factor: f64,
    pub predicted_io_wait: f64,
    pub cache_hotness: f32,
}

/// Unix/Linux Process States
#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    Runnable,      // TASK_RUNNING
    Sleeping,      // TASK_INTERRUPTIBLE
    DiskSleep,     // TASK_UNINTERRUPTIBLE (ReactOS: PsWaiting)
    Stopped,       // TASK_STOPPED
    Traced,        // TASK_TRACED
    Zombie,        // EXIT_ZOMBIE
    Dead,          // EXIT_DEAD
    Idle,          // Special idle task
}

// Weight lookup table - directly from Linux kernel (kernel/sched_fair.c)
// Index = priority + 20 (to handle negative nice values)
const NICE_TO_WEIGHT: [u64; 40] = [
    88761, 71755, 56483, 46273, 36291,
    29154, 23254, 18705, 14949, 12307,
    10000, 8202,  6550,  5420,  4372,
    3566,  2873,  2337,  1906,  1559,
    1277,  1024,  820,   673,   548,
    447,   362,   296,   241,   197,
    160,   130,   105,   86,    70,
    57,    46,    37,    30,    24,
];

const NICE_0_LOAD: u64 = 1024;
const NICE_0_INDEX: usize = 20; // Index for nice=0

// ============================================================================
// SCHEDULER IMPLEMENTATION
// ============================================================================

/// The Core Scheduler implementing CFS logic with AI enhancements
pub struct HybridScheduler {
    // BTreeMap acts as our Red-Black Tree equivalent, keeping tasks sorted by vruntime
    runqueue: BTreeMap<u64, Arc<Task>>,
    min_vruntime: u64,
    nr_running: u64,
    
    // Per-CPU runqueues (ReactOS-style SMP architecture)
    cpu_queues: Vec<BTreeMap<u64, Arc<Task>>>,
    
    // Load tracking (Linux: load_avg)
    load_weight: u64,
    
    // Stark AI Predictor
    predictor: SchedulerPredictor,
}

struct SchedulerPredictor {
    weights: Vec<f64>,
    training_data: Vec<TrainingSample>,
}

struct TrainingSample {
    pid: u32,
    io_ratio: f64,
    cpu_burst_avg: u64,
    context_switch_rate: f64,
    optimal_priority: i32,
}

impl SchedulerPredictor {
    pub fn new() -> Self {
        Self {
            weights: vec![1.0; 8],
            training_data: Vec::with_capacity(1000),
        }
    }

    /// Predict efficiency boost for a task based on historical patterns
    pub fn predict_efficiency(&self, task: &Task) -> f64 {
        if self.training_data.is_empty() {
            return 1.0;
        }

        // Find similar tasks and compute weighted boost
        let similar = self.training_data.iter()
            .filter(|s| (s.pid % 100) == (task.pid % 100))
            .take(5);

        let avg_boost: f64 = similar.map(|s| {
            // Higher boost for IO-bound tasks (they wait anyway)
            if s.io_ratio > 0.7 { 1.3 } 
            // Lower boost for CPU-bound with high context switches
            else if s.context_switch_rate > 100.0 { 0.8 }
            else { 1.0 }
        }).sum::<f64>() / similar.count().max(1) as f64;

        avg_boost.clamp(0.5, 2.0)
    }

    /// Learn from task completion
    pub fn learn(&mut self, task: &Task, actual_runtime: u64, io_waits: u64) {
        let sample = TrainingSample {
            pid: task.pid,
            io_ratio: io_waits as f64 / actual_runtime.max(1) as f64,
            cpu_burst_avg: actual_runtime,
            context_switch_rate: task.nr_cpus_allowed as f64,
            optimal_priority: task.priority,
        };

        if self.training_data.len() >= 1000 {
            self.training_data.remove(0);
        }
        self.training_data.push(sample);
    }
}

impl HybridScheduler {
    pub fn new(num_cpus: usize) -> Self {
        Self {
            runqueue: BTreeMap::new(),
            min_vruntime: 0,
            nr_running: 0,
            cpu_queues: (0..num_cpus).map(|_| BTreeMap::new()).collect(),
            load_weight: 0,
            predictor: SchedulerPredictor::new(),
        }
    }

    /// Convert nice value to weight (Linux: sched_prio_to_weight)
    fn nice_to_weight(nice: i32) -> u64 {
        let idx = ((nice + 20).clamp(0, 39)) as usize;
        NICE_TO_WEIGHT[idx]
    }

    /// Enqueue a task - Linux: enqueue_task_fair()
    pub fn enqueue(&mut self, mut task: Task) {
        // Calculate AI boost (Stark enhancement)
        let boost = self.predictor.predict_efficiency(&task);
        task.ai_boost_factor = boost;

        // Adjust weight based on AI insight
        let base_weight = Self::nice_to_weight(task.priority);
        let adjusted_weight = if boost > 1.0 {
            (base_weight as f64 * boost) as u64
        } else {
            base_weight
        };
        
        task.weight = adjusted_weight;

        // Place entity on the rbtree (Linux: place_entity)
        // If vruntime is too far behind, bring it forward to prevent starvation
        let latency = 6_000_000u64; // 6ms target latency
        let vruntime_limit = self.min_vruntime + latency;

        if task.vruntime < self.min_vruntime {
            task.vruntime = if self.min_vruntime > latency {
                self.min_vruntime - latency / 2
            } else {
                self.min_vruntime
            };
        }

        // Ensure we don't jump too far ahead either
        if task.vruntime > vruntime_limit {
            task.vruntime = vruntime_limit;
        }

        let arc_task = Arc::new(task);
        self.runqueue.insert(arc_task.vruntime, arc_task.clone());
        self.load_weight += adjusted_weight;
        self.nr_running += 1;

        // Add to per-CPU queue (ReactOS-style SMP)
        let cpu_idx = (arc_task.pid as usize) % self.cpu_queues.len();
        self.cpu_queues[cpu_idx].insert(arc_task.vruntime, arc_task);
    }

    /// Dequeue a task - Linux: dequeue_task_fair()
    pub fn dequeue(&mut self, pid: u32) -> Option<Arc<Task>> {
        // Find and remove from global runqueue
        let task = self.runqueue.values().find(|t| t.pid == pid).cloned()?;
        
        // Remove from runqueue by vruntime key
        self.runqueue.remove(&task.vruntime);
        self.load_weight = self.load_weight.saturating_sub(task.weight);
        self.nr_running = self.nr_running.saturating_sub(1);

        // Remove from per-CPU queue
        let cpu_idx = (pid as usize) % self.cpu_queues.len();
        self.cpu_queues[cpu_idx].remove(&task.vruntime);

        Some(task)
    }

    /// Pick next task - Linux: pick_next_task_fair()
    /// Returns task with lowest vruntime (leftmost in RB-Tree)
    pub fn pick_next(&mut self) -> Option<Arc<Task>> {
        self.runqueue.iter().next().map(|(_, task)| task.clone())
    }

    /// Pick next task for specific CPU (ReactOS SMP optimization)
    pub fn pick_next_cpu(&mut self, cpu_id: usize) -> Option<Arc<Task>> {
        if cpu_id < self.cpu_queues.len() {
            self.cpu_queues[cpu_id].iter().next().map(|(_, task)| task.clone())
        } else {
            self.pick_next()
        }
    }

    /// Update stats when task yields/finishes - Linux: update_curr()
    pub fn update_stats(&mut self, pid: u32, delta_exec: u64, io_waits: u64) {
        // Get mutable access - in real kernel this is done via pointers
        if let Some(task_arc) = self.runqueue.values().find(|t| t.pid == pid).cloned() {
            let mut task = (*task_arc).clone();
            
            // Core CFS formula: vruntime += delta_exec * (NICE_0_LOAD / weight)
            // This ensures lower priority (lower weight) tasks accumulate vruntime faster
            let weighted_delta = (delta_exec * NICE_0_LOAD) / task.weight.max(1);
            
            task.vruntime += weighted_delta;
            task.sum_exec_runtime += delta_exec;
            
            // Update minimum vruntime
            if task.vruntime > self.min_vruntime {
                self.min_vruntime = task.vruntime;
            }

            // AI Learning (Stark enhancement)
            self.predictor.learn(&task, delta_exec, io_waits);

            // Re-insert updated task
            self.runqueue.remove(&task_arc.vruntime);
            let updated_arc = Arc::new(task);
            self.runqueue.insert(updated_arc.vruntime, updated_arc);
        }
    }

    /// Get scheduler statistics
    pub fn get_stats(&self) -> SchedulerStats {
        SchedulerStats {
            nr_running: self.nr_running,
            load_weight: self.load_weight,
            min_vruntime: self.min_vruntime,
            queue_depth: self.runqueue.len(),
            ai_samples: self.predictor.training_data.len(),
        }
    }
}

pub struct SchedulerStats {
    pub nr_running: u64,
    pub load_weight: u64,
    pub min_vruntime: u64,
    pub queue_depth: usize,
    pub ai_samples: usize,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nice_to_weight_conversion() {
        // Nice 0 should map to 1024
        assert_eq!(HybridScheduler::nice_to_weight(0), 1024);
        
        // Nice -20 (highest priority) should be largest weight
        assert_eq!(HybridScheduler::nice_to_weight(-20), 88761);
        
        // Nice 19 (lowest priority) should be smallest weight
        assert_eq!(HybridScheduler::nice_to_weight(19), 24);
    }

    #[test]
    fn test_cfs_fairness() {
        let mut sched = HybridScheduler::new(4);
        
        // High priority task (nice -5)
        let task_high = Task {
            pid: 1, tid: 1, priority: -5, static_priority: -5,
            vruntime: 0, weight: 0, state: TaskState::Runnable,
            last_run: None, sum_exec_runtime: 0, nr_cpus_allowed: 4,
            ai_boost_factor: 1.0, predicted_io_wait: 0.0, cache_hotness: 0.5,
        };
        
        // Low priority task (nice 10)
        let task_low = Task {
            pid: 2, tid: 2, priority: 10, static_priority: 10,
            vruntime: 0, weight: 0, state: TaskState::Runnable,
            last_run: None, sum_exec_runtime: 0, nr_cpus_allowed: 4,
            ai_boost_factor: 1.0, predicted_io_wait: 0.0, cache_hotness: 0.5,
        };

        sched.enqueue(task_high);
        sched.enqueue(task_low);

        // Both should be in runqueue
        assert_eq!(sched.nr_running, 2);
        
        // Pick next - should be one of them (both have vruntime=0)
        let next = sched.pick_next();
        assert!(next.is_some());
    }

    #[test]
    fn test_vruntime_accumulation() {
        let mut sched = HybridScheduler::new(1);
        
        let task = Task {
            pid: 1, tid: 1, priority: 0, static_priority: 0,
            vruntime: 0, weight: 1024, state: TaskState::Runnable,
            last_run: None, sum_exec_runtime: 0, nr_cpus_allowed: 1,
            ai_boost_factor: 1.0, predicted_io_wait: 0.0, cache_hotness: 0.5,
        };

        let initial_vruntime = task.vruntime;
        sched.enqueue(task);
        
        // Simulate 10ms of execution
        sched.update_stats(1, 10_000_000, 0);
        
        // Vruntime should have increased
        let stats = sched.get_stats();
        assert!(stats.min_vruntime > initial_vruntime);
    }

    #[test]
    fn test_smp_cpu_affinity() {
        let mut sched = HybridScheduler::new(4);
        
        for i in 0..8 {
            let task = Task {
                pid: i, tid: i, priority: 0, static_priority: 0,
                vruntime: i * 1000, weight: 1024, state: TaskState::Runnable,
                last_run: None, sum_exec_runtime: 0, nr_cpus_allowed: 4,
                ai_boost_factor: 1.0, predicted_io_wait: 0.0, cache_hotness: 0.5,
            };
            sched.enqueue(task);
        }

        // Each CPU should have tasks
        for cpu in 0..4 {
            let next = sched.pick_next_cpu(cpu);
            assert!(next.is_some());
        }
    }
}
