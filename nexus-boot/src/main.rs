#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]

use bootloader::{BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use x86_64::instructions::interrupts;
use nexus_kernel::memory::MemoryManager;
use nexus_kernel::scheduler::Scheduler;
use nexus_telemetry::TelemetryEngine;

const BOOT_CONFIG: BootloaderConfig = BootloaderConfig {
    map_physical_memory: true,
    ..BootloaderConfig::DEFAULT
};

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static mut BootInfo) -> ! {
    // Initialize early telemetry - Stark always monitors from boot
    let mut telemetry = TelemetryEngine::new();
    telemetry.log_event("BOOT", "NEXUS OS initializing...", 0);
    
    // Detect hardware capabilities immediately
    let cpu_features = detect_cpu_features();
    telemetry.log_event("HARDWARE", &format!("CPU Features: {:?}", cpu_features), 1);
    
    // Initialize memory manager with physical map from bootloader
    let phys_map_offset = boot_info.physical_memory_offset;
    let mut memory_manager = MemoryManager::new(phys_map_offset);
    memory_manager.initialize_heap(boot_info);
    telemetry.log_event("MEMORY", "Heap initialized", 2);
    
    // Initialize interrupt descriptor table
    initialize_idt();
    telemetry.log_event("INTERRUPTS", "IDT loaded", 3);
    
    // Initialize scheduler with AI models
    let mut scheduler = Scheduler::new();
    scheduler.load_ai_models();
    telemetry.log_event("SCHEDULER", "AI models loaded", 4);
    
    // Create init process (PID 1)
    scheduler.create_init_process();
    telemetry.log_event("PROCESS", "Init process created", 5);
    
    // Enable interrupts
    unsafe {
        interrupts::enable();
    }
    telemetry.log_event("SYSTEM", "Interrupts enabled", 6);
    
    // Boot complete - hand off to scheduler
    telemetry.log_event("BOOT", "NEXUS OS ready. Let's fly.", 7);
    
    // Enter scheduler loop - never returns
    scheduler.run();
}

fn detect_cpu_features() -> CpuFeatures {
    use x86_64::registers::control::{Cr0, Cr4, Cr4Flags};
    use x86_64::instructions::tables::load_tss;
    
    let mut features = CpuFeatures::empty();
    
    // Check for SSE
    if Cr0::read().contains(Cr0::MONITOR_COPROCESSOR) {
        features.insert(CpuFeatures::SSE);
    }
    
    // Check for SSE2
    if Cr4::read().contains(Cr4Flags::SSE_ENABLE) {
        features.insert(CpuFeatures::SSE2);
    }
    
    // Check for AVX (simplified)
    // In real implementation, would use CPUID instruction
    
    features
}

fn initialize_idt() {
    use x86_64::structures::idt::{InterruptDescriptorTable, HandlerFunc};
    use x86_64::structures::idt::InterruptStackFrame;
    use x86_64::instructions::tables::load_idt;
    use x86_64::addr::VirtAddr;
    
    static mut IDT: Option<InterruptDescriptorTable> = None;
    
    unsafe {
        let mut idt = InterruptDescriptorTable::new();
        
        // Register exception handlers
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(fpu_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.security_exception.set_handler_fn(security_exception_handler);
        
        // Load IDT
        IDT = Some(idt);
        let idt_ptr = VirtAddr::from_ptr(IDT.as_ref().unwrap());
        load_idt(&idt_ptr.to_gdt_entry());
    }
}

extern "x86-interrupt" fn divide_error_handler(_stack_frame: InterruptStackFrame) {
    panic!("Divide by zero error!");
}

extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {
    // Debug exception - can be used for breakpoints
}

extern "x86-interrupt" fn nmi_handler(_stack_frame: InterruptStackFrame) {
    panic!("Non-maskable interrupt!");
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    // Breakpoint hit - useful for debugging
}

extern "x86-interrupt" fn overflow_handler(_stack_frame: InterruptStackFrame) {
    panic!("Overflow exception!");
}

extern "x86-interrupt" fn bound_range_handler(_stack_frame: InterruptStackFrame) {
    panic!("Bound range exceeded!");
}

extern "x86-interrupt" fn invalid_opcode_handler(_stack_frame: InterruptStackFrame) {
    panic!("Invalid opcode!");
}

extern "x86-interrupt" fn device_not_available_handler(_stack_frame: InterruptStackFrame) {
    panic!("Device not available!");
}

extern "x86-interrupt" fn double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("Double fault! System halting.");
}

extern "x86-interrupt" fn invalid_tss_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("Invalid TSS!");
}

extern "x86-interrupt" fn segment_not_present_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("Segment not present!");
}

extern "x86-interrupt" fn stack_segment_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("Stack segment fault!");
}

extern "x86-interrupt" fn general_protection_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("General protection fault! Error code: {}", _error_code);
}

extern "x86-interrupt" fn page_fault_handler(_stack_frame: InterruptStackFrame, error_code: u64) {
    use x86_64::registers::control::Cr2;
    let addr = Cr2::read();
    panic!("Page fault at address {:?}, error code: {:?}", addr, error_code);
}

extern "x86-interrupt" fn fpu_handler(_stack_frame: InterruptStackFrame) {
    panic!("x87 floating point exception!");
}

extern "x86-interrupt" fn alignment_check_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("Alignment check failed!");
}

extern "x86-interrupt" fn machine_check_handler(_stack_frame: InterruptStackFrame) -> ! {
    panic!("Machine check exception! Hardware failure detected.");
}

extern "x86-interrupt" fn simd_handler(_stack_frame: InterruptStackFrame) {
    panic!("SIMD floating point exception!");
}

extern "x86-interrupt" fn virtualization_handler(_stack_frame: InterruptStackFrame) {
    panic!("Virtualization exception!");
}

extern "x86-interrupt" fn security_exception_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("Security exception!");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CpuFeatures: u8 {
    const SSE = 1 << 0;
    const SSE2 = 1 << 1;
    const AVX = 1 << 2;
    const AVX2 = 1 << 3;
    const AVX512 = 1 << 4;
}

impl CpuFeatures {
    fn empty() -> Self {
        CpuFeatures { bits: 0 }
    }
    
    fn insert(&mut self, other: CpuFeatures) {
        self.bits |= other.bits;
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interrupts::disable();
    loop {
        // In real implementation, would print to screen/serial
        // For now, just halt
        x86_64::instructions::hlt();
    }
}
