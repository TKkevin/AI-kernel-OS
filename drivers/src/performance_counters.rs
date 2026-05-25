//! Performance Counter Management

pub struct PerformanceCounter {
    pub name: String,
    pub value: u64,
    pub enabled: bool,
}

pub struct PerformanceMonitor {
    counters: Vec<PerformanceCounter>,
    sampling_interval_ns: u64,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        PerformanceMonitor {
            counters: vec![
                PerformanceCounter { name: "cpu_cycles".to_string(), value: 0, enabled: true },
                PerformanceCounter { name: "instructions_retired".to_string(), value: 0, enabled: true },
                PerformanceCounter { name: "cache_misses".to_string(), value: 0, enabled: true },
                PerformanceCounter { name: "branch_mispredictions".to_string(), value: 0, enabled: true },
                PerformanceCounter { name: "tlb_misses".to_string(), value: 0, enabled: true },
            ],
            sampling_interval_ns: 1_000_000,
        }
    }

    pub fn read_counter(&self, name: &str) -> Option<u64> {
        self.counters.iter().find(|c| c.name == name).map(|c| c.value)
    }

    pub fn enable_counter(&mut self, name: &str) {
        if let Some(counter) = self.counters.iter_mut().find(|c| c.name == name) {
            counter.enabled = true;
        }
    }

    pub fn disable_counter(&mut self, name: &str) {
        if let Some(counter) = self.counters.iter_mut().find(|c| c.name == name) {
            counter.enabled = false;
        }
    }

    pub fn snapshot(&self) -> Vec<(String, u64)> {
        self.counters.iter().filter(|c| c.enabled).map(|c| (c.name.clone(), c.value)).collect()
    }

    pub fn calculate_ipc(&self) -> Option<f32> {
        let cycles = self.read_counter("cpu_cycles")?;
        let instructions = self.read_counter("instructions_retired")?;
        if cycles == 0 { return None; }
        Some(instructions as f32 / cycles as f32)
    }

    pub fn calculate_cache_miss_rate(&self) -> Option<f32> {
        let misses = self.read_counter("cache_misses")?;
        let instructions = self.read_counter("instructions_retired")?;
        if instructions == 0 { return None; }
        Some(misses as f32 / instructions as f32 * 100.0)
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor() {
        let mut pm = PerformanceMonitor::new();
        assert_eq!(pm.read_counter("cpu_cycles"), Some(0));
        pm.disable_counter("tlb_misses");
        let snapshot = pm.snapshot();
        assert!(!snapshot.iter().any(|(name, _)| name == "tlb_misses"));
    }

    #[test]
    fn test_ipc_calculation() {
        let pm = PerformanceMonitor::new();
        assert_eq!(pm.calculate_ipc(), None);
    }
}
