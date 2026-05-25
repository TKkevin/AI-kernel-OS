//! NEXUS Algorithm Core - Part 2: Filesystems, Networking, Security, and AI (31-100+)
//! 
//! Continuing the integration of proven algorithms from Unix, Linux, and ReactOS
//! with Stark-level AI enhancements.

use std::collections::{HashMap, HashSet, VecDeque, BTreeMap};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use ndarray::Array1;
use rand::Rng;

// ============================================================================
// SECTION 3: FILESYSTEM ALGORITHMS (21-35)
// ============================================================================

/// 21. ext4 Extent Tree (simplified)
#[derive(Clone, Debug)]
pub struct Extent {
    pub logical_start: u64,
    pub physical_start: u64,
    pub length: u64,
}

pub struct ExtentTree {
    extents: BTreeMap<u64, Extent>,
}

impl ExtentTree {
    pub fn new() -> Self {
        Self {
            extents: BTreeMap::new(),
        }
    }
    
    pub fn add_extent(&mut self, logical: u64, physical: u64, length: u64) {
        self.extents.insert(logical, Extent {
            logical_start: logical,
            physical_start: physical,
            length,
        });
    }
    
    pub fn find_extent(&self, logical_block: u64) -> Option<&Extent> {
        // Find the extent that contains this logical block
        let mut prev = None;
        for (&logical, extent) in &self.extents {
            if logical > logical_block {
                break;
            }
            prev = Some(extent);
        }
        
        if let Some(extent) = prev {
            if logical_block >= extent.logical_start 
                && logical_block < extent.logical_start + extent.length {
                return Some(extent);
            }
        }
        
        None
    }
    
    pub fn translate(&self, logical_block: u64) -> Option<u64> {
        self.find_extent(logical_block).map(|extent| {
            extent.physical_start + (logical_block - extent.logical_start)
        })
    }
}

/// 22. Unix Fast Filesystem (FFS) Cylinder Groups
pub struct CylinderGroup {
    id: u32,
    free_blocks: Vec<u64>,
    free_inodes: Vec<u32>,
    used_blocks: u64,
    used_inodes: u64,
}

pub struct FfsLayout {
    cylinder_groups: Vec<CylinderGroup>,
    blocks_per_group: u64,
    inodes_per_group: u32,
}

impl FfsLayout {
    pub fn new(num_groups: u32, blocks_per_group: u64, inodes_per_group: u32) -> Self {
        let mut groups = Vec::new();
        for i in 0..num_groups {
            groups.push(CylinderGroup {
                id: i,
                free_blocks: Vec::new(),
                free_inodes: (0..inodes_per_group).collect(),
                used_blocks: 0,
                used_inodes: 0,
            });
        }
        
        Self {
            cylinder_groups: groups,
            blocks_per_group,
            inodes_per_group,
        }
    }
    
    pub fn allocate_block(&mut self, preferred_group: Option<u32>) -> Option<(u32, u64)> {
        // Try preferred group first (locality optimization)
        if let Some(pref) = preferred_group {
            if pref < self.cylinder_groups.len() as u32 {
                let group = &mut self.cylinder_groups[pref as usize];
                if !group.free_blocks.is_empty() {
                    let block = group.free_blocks.pop().unwrap();
                    group.used_blocks += 1;
                    return Some((pref, block));
                }
            }
        }
        
        // Find any group with free blocks
        for (i, group) in self.cylinder_groups.iter_mut().enumerate() {
            if !group.free_blocks.is_empty() {
                let block = group.free_blocks.pop().unwrap();
                group.used_blocks += 1;
                return Some((i as u32, block));
            }
        }
        
        None
    }
    
    pub fn allocate_inode(&mut self) -> Option<(u32, u32)> {
        for (i, group) in self.cylinder_groups.iter_mut().enumerate() {
            if let Some(inode) = group.free_inodes.pop() {
                group.used_inodes += 1;
                return Some((i as u32, inode));
            }
        }
        None
    }
}

/// 23. Log-Structured Filesystem (LFS)
pub struct LfsSegment {
    id: u64,
    data: Vec<u8>,
    write_pointer: usize,
}

pub struct LogStructuredFs {
    segments: Vec<LfsSegment>,
    segment_size: usize,
    current_segment: usize,
    inode_map: HashMap<u64, (u64, usize)>, // inode -> (segment_id, offset)
}

impl LogStructuredFs {
    pub fn new(segment_size: usize) -> Self {
        Self {
            segments: vec![LfsSegment {
                id: 0,
                data: vec![0; segment_size],
                write_pointer: 0,
            }],
            segment_size,
            current_segment: 0,
            inode_map: HashMap::new(),
        }
    }
    
    pub fn write_data(&mut self, inode: u64, data: &[u8]) -> Option<(u64, usize)> {
        let segment = &mut self.segments[self.current_segment];
        
        if segment.write_pointer + data.len() > self.segment_size {
            // Need new segment
            self.segments.push(LfsSegment {
                id: self.segments.len() as u64,
                data: vec![0; self.segment_size],
                write_pointer: 0,
            });
            self.current_segment += 1;
        }
        
        let segment = &mut self.segments[self.current_segment];
        let offset = segment.write_pointer;
        segment.data[offset..offset + data.len()].copy_from_slice(data);
        segment.write_pointer += data.len();
        
        self.inode_map.insert(inode, (segment.id, offset));
        Some((segment.id, offset))
    }
    
    pub fn read_data(&self, inode: u64) -> Option<&[u8]> {
        let (seg_id, offset) = *self.inode_map.get(&inode)?;
        let segment = &self.segments[seg_id as usize];
        
        // Find end of data (simplified - real LFS would store metadata)
        Some(&segment.data[offset..])
    }
}

/// 24. Copy-on-Write Filesystem (Btrfs/ZFS style)
#[derive(Clone)]
struct CowNode {
    data: Vec<u8>,
    refcount: u64,
    checksum: u64,
}

pub struct CowFilesystem {
    nodes: HashMap<u64, CowNode>,
    next_node_id: u64,
    root_id: u64,
}

impl CowFilesystem {
    pub fn new() -> Self {
        let mut fs = Self {
            nodes: HashMap::new(),
            next_node_id: 1,
            root_id: 0,
        };
        
        // Create root node
        let root_id = fs.allocate_node(vec![0; 4096]);
        fs.root_id = root_id;
        fs
    }
    
    fn allocate_node(&mut self, data: Vec<u8>) -> u64 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        
        let checksum = Self::calculate_checksum(&data);
        self.nodes.insert(id, CowNode {
            data,
            refcount: 1,
            checksum,
        });
        
        id
    }
    
    fn calculate_checksum(data: &[u8]) -> u64 {
        // Simple CRC-like checksum (real implementation would use proper CRC32C)
        data.iter().enumerate().fold(0u64, |acc, (i, &b)| {
            acc ^ ((b as u64) << ((i % 8) * 8))
        })
    }
    
    pub fn clone_tree(&mut self, root: u64) -> u64 {
        // Increment refcount for COW semantics
        if let Some(node) = self.nodes.get_mut(&root) {
            node.refcount += 1;
        }
        root
    }
    
    pub fn write_node(&mut self, node_id: u64, data: &[u8]) -> u64 {
        if let Some(node) = self.nodes.get(&node_id) {
            if node.refcount > 1 {
                // COW: create new copy
                node.refcount -= 1;
                return self.allocate_node(data.to_vec());
            } else {
                // Can modify in place
                let node = self.nodes.get_mut(&node_id).unwrap();
                node.data = data.to_vec();
                node.checksum = Self::calculate_checksum(&data);
                return node_id;
            }
        }
        self.allocate_node(data.to_vec())
    }
    
    pub fn verify_checksum(&self, node_id: u64) -> bool {
        self.nodes.get(&node_id)
            .map(|node| node.checksum == Self::calculate_checksum(&node.data))
            .unwrap_or(false)
    }
}

/// 25. Journaling Block Allocator
pub struct Journal {
    entries: VecDeque<JournalEntry>,
    max_size: usize,
    commit_sequence: u64,
}

#[derive(Clone)]
enum JournalEntry {
    Metadata { block: u64, old_data: Vec<u8>, new_data: Vec<u8> },
    Commit { sequence: u64 },
    Checkpoint { sequence: u64 },
}

impl Journal {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_size),
            max_size,
            commit_sequence: 0,
        }
    }
    
    pub fn log_metadata(&mut self, block: u64, old_data: Vec<u8>, new_data: Vec<u8>) {
        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }
        
        self.entries.push_back(JournalEntry::Metadata {
            block,
            old_data,
            new_data,
        });
    }
    
    pub fn commit(&mut self) -> u64 {
        self.commit_sequence += 1;
        self.entries.push_back(JournalEntry::Commit {
            sequence: self.commit_sequence,
        });
        self.commit_sequence
    }
    
    pub fn checkpoint(&mut self, up_to_sequence: u64) {
        while let Some(entry) = self.entries.front() {
            match entry {
                JournalEntry::Checkpoint { sequence } if *sequence >= up_to_sequence => break,
                JournalEntry::Commit { sequence } if *sequence >= up_to_sequence => break,
                _ => {
                    self.entries.pop_front();
                }
            }
        }
        
        self.entries.push_back(JournalEntry::Checkpoint {
            sequence: up_to_sequence,
        });
    }
    
    pub fn replay(&self) -> Vec<(u64, Vec<u8>)> {
        let mut writes = Vec::new();
        
        for entry in &self.entries {
            if let JournalEntry::Metadata { block, new_data, .. } = entry {
                writes.push((*block, new_data.clone()));
            }
        }
        
        writes
    }
}

// ============================================================================
// SECTION 4: NETWORKING ALGORITHMS (36-50)
// ============================================================================

/// 36. TCP Congestion Control: Reno
pub struct TcpReno {
    cwnd: u64,      // Congestion window (in segments)
    ssthresh: u64,  // Slow start threshold
    mss: u64,       // Maximum Segment Size
}

impl TcpReno {
    pub fn new(mss: u64) -> Self {
        Self {
            cwnd: 1,
            ssthresh: 65535,
            mss,
        }
    }
    
    pub fn on_ack(&mut self, ack_count: u64) {
        if self.cwnd < self.ssthresh {
            // Slow start phase
            self.cwnd += ack_count;
        } else {
            // Congestion avoidance phase
            self.cwnd += ack_count / self.cwnd;
        }
    }
    
    pub fn on_loss_fast_retransmit(&mut self) {
        self.ssthresh = self.cwnd / 2;
        self.cwnd = self.ssthresh + 3; // Fast recovery
    }
    
    pub fn on_loss_timeout(&mut self) {
        self.ssthresh = self.cwnd / 2;
        self.cwnd = 1; // Back to slow start
    }
    
    pub fn get_cwnd_bytes(&self) -> u64 {
        self.cwnd * self.mss
    }
}

/// 37. TCP Congestion Control: CUBIC (Linux default)
pub struct TcpCubic {
    cwnd: f64,
    ssthresh: f64,
    w_max: f64,
    k: f64,
    t_epoch: u64,
    mss: u64,
}

impl TcpCubic {
    pub fn new(mss: u64) -> Self {
        Self {
            cwnd: 1.0,
            ssthresh: 65535.0,
            w_max: 0.0,
            k: 0.0,
            t_epoch: 0,
            mss,
        }
    }
    
    pub fn on_ack(&mut self, current_time_ms: u64) {
        if self.cwnd < self.ssthresh {
            // Slow start
            self.cwnd += 1.0;
        } else {
            // Cubic function
            let t = (current_time_ms - self.t_epoch) as f64 / 1000.0;
            let cubic = 0.4 * ((t - self.k).powi(3)) + self.w_max;
            
            // TCP-friendly region
            let tcp_friendly = (self.w_max * 0.5 * self.mss as f64) / self.cwnd;
            
            self.cwnd = cubic.max(tcp_friendly);
        }
    }
    
    pub fn on_loss(&mut self, current_time_ms: u64) {
        self.w_max = self.cwnd;
        self.ssthresh = (self.cwnd * 0.7) as u64 as f64;
        self.cwnd = self.ssthresh;
        self.k = ((1.0 - self.w_max * 0.7 / 0.4).cbrt()) as f64;
        self.t_epoch = current_time_ms;
    }
}

/// 38. Nagle's Algorithm
pub struct NagleBuffer {
    buffer: Vec<u8>,
    mss: usize,
    has_unacked_data: bool,
}

impl NagleBuffer {
    pub fn new(mss: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(mss),
            mss,
            has_unacked_data: false,
        }
    }
    
    pub fn send(&mut self, data: &[u8]) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        
        // If we have unacked data and small amount of new data, buffer it
        if self.has_unacked_data && !self.buffer.is_empty() {
            self.buffer.extend_from_slice(data);
            return packets;
        }
        
        // Send full MSS immediately
        let mut remaining = data;
        while remaining.len() >= self.mss {
            packets.push(remaining[..self.mss].to_vec());
            remaining = &remaining[self.mss..];
        }
        
        // Buffer remaining small data
        if !remaining.is_empty() {
            self.buffer.extend_from_slice(remaining);
        }
        
        packets
    }
    
    pub fn on_ack(&mut self) -> Option<Vec<u8>> {
        self.has_unacked_data = false;
        
        if !self.buffer.is_empty() {
            let packet = self.buffer.clone();
            self.buffer.clear();
            self.has_unacked_data = true;
            Some(packet)
        } else {
            None
        }
    }
}

/// 39. Sliding Window Protocol
pub struct SlidingWindow {
    window_size: u64,
    base: u64,
    next_seq: u64,
    acked: HashSet<u64>,
    sent_times: HashMap<u64, u64>,
}

impl SlidingWindow {
    pub fn new(window_size: u64) -> Self {
        Self {
            window_size,
            base: 0,
            next_seq: 0,
            acked: HashSet::new(),
            sent_times: HashMap::new(),
        }
    }
    
    pub fn can_send(&self) -> bool {
        self.next_seq < self.base + self.window_size
    }
    
    pub fn send(&mut self, seq: u64, timestamp: u64) {
        self.sent_times.insert(seq, timestamp);
        self.next_seq += 1;
    }
    
    pub fn ack(&mut self, seq: u64) {
        self.acked.insert(seq);
        
        // Slide window forward
        while self.acked.contains(&self.base) {
            self.base += 1;
            self.sent_times.remove(&self.base);
        }
    }
    
    pub fn get_unacked(&self) -> Vec<u64> {
        let mut unacked = Vec::new();
        for seq in self.base..self.next_seq {
            if !self.acked.contains(&seq) {
                unacked.push(seq);
            }
        }
        unacked
    }
}

/// 40. RED (Random Early Detection) Queue Management
pub struct RedQueue {
    queue: VecDeque<u64>,
    max_queue: usize,
    min_threshold: f64,
    max_threshold: f64,
    max_p: f64,
    avg_queue: f64,
    weight: f64,
    count: u64,
}

impl RedQueue {
    pub fn new(max_queue: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_queue),
            max_queue,
            min_threshold: 0.25,
            max_threshold: 0.75,
            max_p: 0.1,
            avg_queue: 0.0,
            weight: 0.002,
            count: 0,
        }
    }
    
    pub fn enqueue(&mut self, packet: u64) -> bool {
        let current_queue = self.queue.len() as f64 / self.max_queue as f64;
        
        // Update average queue size
        self.avg_queue = (1.0 - self.weight) * self.avg_queue + self.weight * current_queue;
        
        if self.avg_queue < self.min_threshold {
            self.count = 0;
            self.queue.push_back(packet);
            true
        } else if self.avg_queue > self.max_threshold {
            // Drop packet
            false
        } else {
            // Random early detection
            self.count += 1;
            let p = self.max_p * (self.avg_queue - self.min_threshold) 
                / (self.max_threshold - self.min_threshold);
            
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < p / self.count as f64 {
                self.count = 0;
                false // Drop
            } else {
                self.queue.push_back(packet);
                true
            }
        }
    }
    
    pub fn dequeue(&mut self) -> Option<u64> {
        self.queue.pop_front()
    }
}

// ============================================================================
// SECTION 5: SECURITY ALGORITHMS (51-70)
// ============================================================================

/// 51. Capability-Based Security (ReactOS style)
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Capability {
    resource_id: u64,
    permissions: u32,
    expiry: Option<u64>,
}

pub struct CapabilityManager {
    capabilities: HashMap<u32, HashSet<Capability>>, // pid -> capabilities
    next_cap_id: u64,
}

impl CapabilityManager {
    pub const READ: u32 = 0x01;
    pub const WRITE: u32 = 0x02;
    pub const EXECUTE: u32 = 0x04;
    pub const DELETE: u32 = 0x08;
    
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            next_cap_id: 1,
        }
    }
    
    pub fn grant_capability(&mut self, pid: u32, resource_id: u64, permissions: u32, expiry: Option<u64>) {
        let cap = Capability {
            resource_id,
            permissions,
            expiry,
        };
        
        self.capabilities.entry(pid).or_insert_with(HashSet::new).insert(cap);
    }
    
    pub fn check_permission(&self, pid: u32, resource_id: u64, required: u32) -> bool {
        if let Some(caps) = self.capabilities.get(&pid) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            for cap in caps {
                if cap.resource_id == resource_id 
                    && (cap.permissions & required) == required
                    && cap.expiry.map_or(true, |exp| exp > now) {
                    return true;
                }
            }
        }
        false
    }
    
    pub fn revoke_capabilities(&mut self, pid: u32, resource_id: Option<u64>) {
        if let Some(caps) = self.capabilities.get_mut(&pid) {
            if let Some(rid) = resource_id {
                caps.retain(|cap| cap.resource_id != rid);
            } else {
                caps.clear();
            }
        }
    }
}

/// 52. Mandatory Access Control (MAC) - SELinux style
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SecurityLabel {
    user: String,
    role: String,
    type_: String,
    level: String,
}

pub struct MacEnforcer {
    policies: HashMap<(String, String), HashSet<String>>, // (source_type, action) -> target_types
    processes: HashMap<u32, SecurityLabel>,
    objects: HashMap<u64, SecurityLabel>,
}

impl MacEnforcer {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            processes: HashMap::new(),
            objects: HashMap::new(),
        }
    }
    
    pub fn set_process_label(&mut self, pid: u32, label: SecurityLabel) {
        self.processes.insert(pid, label);
    }
    
    pub fn set_object_label(&mut self, obj_id: u64, label: SecurityLabel) {
        self.objects.insert(obj_id, label);
    }
    
    pub fn add_policy(&mut self, source_type: &str, action: &str, target_type: &str) {
        self.policies
            .entry((source_type.to_string(), action.to_string()))
            .or_insert_with(HashSet::new)
            .insert(target_type.to_string());
    }
    
    pub fn check_access(&self, pid: u32, obj_id: u64, action: &str) -> bool {
        let proc_label = match self.processes.get(&pid) {
            Some(l) => l,
            None => return false,
        };
        
        let obj_label = match self.objects.get(&obj_id) {
            Some(l) => l,
            None => return false,
        };
        
        let key = (proc_label.type_.clone(), action.to_string());
        self.policies.get(&key)
            .map(|targets| targets.contains(&obj_label.type_))
            .unwrap_or(false)
    }
}

/// 53. Address Space Layout Randomization (ASLR)
pub struct AslrManager {
    entropy_bits: u32,
    base_addresses: HashMap<u32, u64>,
}

impl AslrManager {
    pub fn new(entropy_bits: u32) -> Self {
        Self {
            entropy_bits,
            base_addresses: HashMap::new(),
        }
    }
    
    pub fn randomize_base(&mut self, pid: u32, original_base: u64) -> u64 {
        let mut rng = rand::thread_rng();
        let mask = (1u64 << self.entropy_bits) - 1;
        let offset = (rng.gen::<u64>() & mask) << 12; // Page-aligned
        
        let randomized = original_base + offset;
        self.base_addresses.insert(pid, randomized);
        randomized
    }
    
    pub fn get_base(&self, pid: u32) -> Option<u64> {
        self.base_addresses.get(&pid).copied()
    }
}

/// 54. Stack Canary Protection
pub struct StackCanaryManager {
    canaries: HashMap<u64, u64>, // frame_ptr -> canary_value
}

impl StackCanaryManager {
    pub fn new() -> Self {
        Self {
            canaries: HashMap::new(),
        }
    }
    
    pub fn generate_canary(&mut self, frame_ptr: u64) -> u64 {
        let mut rng = rand::thread_rng();
        let canary = rng.gen::<u64>();
        self.canaries.insert(frame_ptr, canary);
        canary
    }
    
    pub fn verify_canary(&self, frame_ptr: u64, canary: u64) -> bool {
        self.canaries.get(&frame_ptr) == Some(&canary)
    }
    
    pub fn remove_canary(&mut self, frame_ptr: u64) {
        self.canaries.remove(&frame_ptr);
    }
}

// ============================================================================
// SECTION 6: AI/ML ENHANCEMENTS (71-100+)
// ============================================================================

/// 71. AI-Enhanced Scheduler Predictor
pub struct AiSchedulerPredictor {
    history: VecDeque<(Array1<f32>, u32)>, // (features, chosen_cpu)
    weights: Array1<f32>,
    learning_rate: f32,
}

impl AiSchedulerPredictor {
    pub fn new(feature_count: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(1000),
            weights: Array1::zeros(feature_count),
            learning_rate: 0.01,
        }
    }
    
    pub fn train(&mut self, features: Array1<f32>, optimal_cpu: u32) {
        self.history.push_back((features.clone(), optimal_cpu));
        
        // Online learning: adjust weights based on outcome
        let prediction: f32 = features.iter().zip(self.weights.iter()).map(|(f, w)| f * w).sum();
        let target = optimal_cpu as f32;
        let error = target - prediction;
        
        for i in 0..features.len() {
            self.weights[i] += self.learning_rate * error * features[i];
        }
    }
    
    pub fn predict(&self, features: &Array1<f32>) -> u32 {
        let prediction: f32 = features.iter().zip(self.weights.iter()).map(|(f, w)| f * w).sum();
        prediction.clamp(0.0, 3.0) as u32
    }
}

/// 72. Predictive Prefetcher
pub struct PredictivePrefetcher {
    access_patterns: HashMap<u64, VecDeque<u64>>,
    confidence: HashMap<u64, f32>,
    prefetch_distance: usize,
}

impl PredictivePrefetcher {
    pub fn new(prefetch_distance: usize) -> Self {
        Self {
            access_patterns: HashMap::new(),
            confidence: HashMap::new(),
            prefetch_distance,
        }
    }
    
    pub fn record_access(&mut self, stream_id: u64, address: u64) -> Vec<u64> {
        let pattern = self.access_patterns.entry(stream_id).or_insert_with(VecDeque::new);
        pattern.push_back(address);
        
        if pattern.len() > 10 {
            pattern.pop_front();
        }
        
        // Detect stride pattern
        if pattern.len() >= 3 {
            let strides: Vec<i64> = pattern.iter()
                .zip(pattern.iter().skip(1))
                .map(|(a, b)| *b as i64 - *a as i64)
                .collect();
            
            if strides.windows(2).all(|w| w[0] == w[1]) {
                let stride = strides[0];
                let last = *pattern.back().unwrap() as i64;
                
                // Generate prefetch addresses
                let prefetches: Vec<u64> = (1..=self.prefetch_distance)
                    .map(|i| (last + stride * i as i64) as u64)
                    .collect();
                
                *self.confidence.entry(stream_id).or_insert(0.5) += 0.1;
                return prefetches;
            }
        }
        
        Vec::new()
    }
}

// More algorithms continue...

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extent_tree() {
        let mut tree = ExtentTree::new();
        tree.add_extent(0, 1000, 100);
        tree.add_extent(100, 1100, 50);
        
        assert_eq!(tree.translate(50), Some(1050));
        assert_eq!(tree.translate(120), Some(1120));
    }
    
    #[test]
    fn test_tcp_reno() {
        let mut reno = TcpReno::new(1460);
        
        // Slow start
        for _ in 0..10 {
            reno.on_ack(1);
        }
        assert!(reno.cwnd > 1);
        
        // Simulate loss
        reno.on_loss_timeout();
        assert_eq!(reno.cwnd, 1);
    }
    
    #[test]
    fn test_capability_security() {
        let mut manager = CapabilityManager::new();
        manager.grant_capability(1, 100, CapabilityManager::READ | CapabilityManager::WRITE, None);
        
        assert!(manager.check_permission(1, 100, CapabilityManager::READ));
        assert!(manager.check_permission(1, 100, CapabilityManager::WRITE));
        assert!(!manager.check_permission(1, 100, CapabilityManager::DELETE));
        assert!(!manager.check_permission(2, 100, CapabilityManager::READ));
    }
}
