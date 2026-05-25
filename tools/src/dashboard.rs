//! NEXUS Real-Time Telemetry Dashboard
//! 
//! A terminal-based dashboard displaying live system metrics.
//! Stark would demand nothing less than complete visibility.

use nexus_telemetry::{TelemetryCollector, SystemMetrics, TelemetryStats};
use termion::{color, style};
use std::io::{stdout, Write};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Dashboard {
    collector: TelemetryCollector,
    width: u16,
    height: u16,
}

impl Dashboard {
    pub fn new() -> Self {
        let (width, height) = termion::terminal_size().unwrap_or((80, 24));
        Dashboard {
            collector: TelemetryCollector::new(100),
            width,
            height,
        }
    }

    /// Simulate incoming telemetry data
    pub fn simulate_metrics(&mut self) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simulate realistic system behavior
        let metrics = SystemMetrics {
            timestamp,
            cpu_utilization: (timestamp % 100) as f32 / 100.0 * 0.8 + 0.1,
            memory_used: 2_000_000_000 + (timestamp % 500_000_000),
            memory_total: 8_000_000_000,
            io_read_ops: timestamp * 100,
            io_write_ops: timestamp * 50,
            context_switches: timestamp * 1000,
            page_faults: timestamp * 10,
            interrupt_count: timestamp * 500,
        };

        self.collector.collect(metrics);
    }

    /// Render the dashboard
    pub fn render(&self) -> String {
        let stats = self.collector.get_stats();
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "{}{}╔══════════════════════════════════════════════════════════╗\n",
            color::Fg(color::Cyan),
            style::Bold
        ));
        output.push_str(&format!(
            "║  {}NEXUS{} Telemetry Dashboard                    ║\n",
            color::Fg(color::White),
            color::Fg(color::Cyan)
        ));
        output.push_str(&format!(
            "║  Real-time System Intelligence                    ║\n"
        ));
        output.push_str(&format!(
            "╚══════════════════════════════════════════════════════════╝{}\n\n",
            style::Reset
        ));

        // CPU Section
        output.push_str(&format!(
            "{}┌────────────────────────────────────────────────────┐\n",
            color::Fg(color::Green)
        ));
        output.push_str(&format!("│  {}CPU UTILIZATION{}                                │\n", 
            color::Fg(color::Yellow), color::Fg(color::Green)));
        output.push_str(&format!("│                                                      │\n"));
        output.push_str(&format!("│  Current:  {:>6.2}%                                  │\n", 
            stats.avg_cpu_utilization * 100.0));
        output.push_str(&format!("│  Samples:  {:>6}                                     │\n", 
            stats.sample_count));
        
        // CPU Bar visualization
        let bar_width = 40;
        let filled = ((stats.avg_cpu_utilization * 100.0 / 100.0) * bar_width as f32) as usize;
        let bar = "█".repeat(filled.min(bar_width)) + &"░".repeat(bar_width - filled.min(bar_width));
        output.push_str(&format!("│  [{}{}]  │\n", 
            color::Fg(color::Red), bar));
        output.push_str(&format!("└────────────────────────────────────────────────────┘{}\n\n", 
            style::Reset));

        // Memory Section
        output.push_str(&format!(
            "{}┌────────────────────────────────────────────────────┐\n",
            color::Fg(color::Blue)
        ));
        output.push_str(&format!("│  {}MEMORY STATUS{}                                    │\n", 
            color::Fg(color::Yellow), color::Fg(color::Blue)));
        output.push_str(&format!("│                                                      │\n"));
        output.push_str(&format!("│  Used:   {:>10} MB                              │\n", 
            stats.peak_memory / 1_000_000));
        output.push_str(&format!("│  Total:  {:>10} MB                              │\n", 
            8000));
        
        let mem_percent = (stats.peak_memory as f32 / 8_000_000_000.0) * 100.0;
        let mem_filled = ((mem_percent / 100.0) * bar_width as f32) as usize;
        let mem_bar = "█".repeat(mem_filled.min(bar_width)) + &"░".repeat(bar_width - mem_filled.min(bar_width));
        output.push_str(&format!("│  [{}{}]  │\n", 
            color::Fg(color::Magenta), mem_bar));
        output.push_str(&format!("└────────────────────────────────────────────────────┘{}\n\n", 
            style::Reset));

        // Activity Metrics
        output.push_str(&format!(
            "{}┌────────────────────────────────────────────────────┐\n",
            color::Fg(color::White)
        ));
        output.push_str(&format!("│  {}SYSTEM ACTIVITY{}                                  │\n", 
            color::Fg(color::Yellow), color::Fg(color::White)));
        output.push_str(&format!("│                                                      │\n"));
        output.push_str(&format!("│  Context Switches:  {:>12}                      │\n", 
            stats.sample_count as u64 * 1000));
        output.push_str(&format!("│  Page Faults:       {:>12}                      │\n", 
            stats.sample_count as u64 * 10));
        output.push_str(&format!("│  Interrupts:        {:>12}                      │\n", 
            stats.sample_count as u64 * 500));
        output.push_str(&format!("└────────────────────────────────────────────────────┘{}\n\n", 
            style::Reset));

        // Footer
        output.push_str(&format!(
            "{}{}  Press Ctrl+C to exit | Updates every 100ms{}",
            color::Fg(color::DarkGray),
            style::Italic,
            style::Reset
        ));

        output
    }

    /// Clear screen and position cursor
    pub fn clear_screen() {
        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
        stdout().flush().unwrap();
    }
}

fn main() {
    let mut dashboard = Dashboard::new();
    
    println!("{}{}Starting NEXUS Telemetry Dashboard...{}", 
        color::Fg(color::Cyan), style::Bold, style::Reset);
    println!("\nInitializing telemetry collection...\n");
    
    // Collect some initial samples
    for _ in 0..10 {
        dashboard.simulate_metrics();
    }
    
    // Render once for demo
    Dashboard::clear_screen();
    println!("{}", dashboard.render());
    
    println!("\n\n{}Dashboard rendering complete.{}", 
        color::Fg(color::Green), style::Reset);
    println!("{}In production, this would update in real-time.{}", 
        color::Fg(color::Yellow));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let dashboard = Dashboard::new();
        assert!(dashboard.width >= 80);
        assert!(dashboard.height >= 24);
    }

    #[test]
    fn test_metrics_collection() {
        let mut dashboard = Dashboard::new();
        dashboard.simulate_metrics();
        let stats = dashboard.collector.get_stats();
        assert_eq!(stats.sample_count, 1);
    }

    #[test]
    fn test_render_output() {
        let dashboard = Dashboard::new();
        let output = dashboard.render();
        assert!(output.contains("NEXUS"));
        assert!(output.contains("CPU UTILIZATION"));
        assert!(output.contains("MEMORY STATUS"));
    }
}
