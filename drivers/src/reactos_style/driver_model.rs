//! ReactOS-Style Driver Architecture
//! 
//! Implements ReactOS-inspired driver model:
//! - IRP (I/O Request Packet) based communication
//! - Driver objects and device objects
//! - Major function dispatch tables
//! - Plug-and-Play manager integration

use std::collections::HashMap;
use std::sync::Arc;

/// I/O Request Packet - Core of ReactOS driver model
#[derive(Debug, Clone)]
pub struct Irp {
    pub major_function: MajorFunction,
    pub minor_function: u32,
    pub io_status: IoStatusBlock,
    pub associated_irp: Option<Arc<Irp>>,
    pub user_buffer: Vec<u8>,
    pub parameters: IrpParameters,
    pub cancel: bool,
    pub cancel_reason: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MajorFunction {
    Create = 0,
    CreateNamedPipe = 1,
    Close = 2,
    Read = 3,
    Write = 4,
    QueryInformation = 5,
    SetInformation = 6,
    QueryEa = 7,
    SetEa = 8,
    FlushBuffers = 9,
    QueryVolumeInformation = 10,
    SetVolumeInformation = 11,
    DirectoryControl = 12,
    FileSystemControl = 13,
    DeviceIoControl = 14,
    InternalDeviceIoControl = 15,
    Shutdown = 16,
    LockControl = 17,
    Cleanup = 18,
    CreateMailslot = 19,
    QuerySecurity = 20,
    SetSecurity = 21,
    Power = 22,
    SystemControl = 23,
    DeviceChange = 24,
    QueryQuota = 25,
    SetQuota = 26,
    Pnp = 27,
}

#[derive(Debug, Clone)]
pub struct IoStatusBlock {
    pub status: NtStatus,
    pub information: usize,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NtStatus {
    Success = 0x00000000,
    Pending = 0x00000103,
    InvalidParameter = 0xC00000EF,
    NoSuchDevice = 0xC000000E,
    AccessDenied = 0xC0000022,
    BufferOverflow = 0x80000005,
    EndOfFile = 0xC0000011,
}

#[derive(Debug, Clone)]
pub enum IrpParameters {
    Create(CreateParameters),
    Read(ReadParameters),
    Write(WriteParameters),
    DeviceControl(DeviceControlParameters),
    Pnp(PnpParameters),
    Power(PowerParameters),
    Other,
}

#[derive(Debug, Clone)]
pub struct CreateParameters {
    pub desired_access: u32,
    pub file_attributes: u32,
    pub share_access: u32,
    pub create_disposition: u32,
    pub create_options: u32,
    pub ea_length: u32,
}

#[derive(Debug, Clone)]
pub struct ReadParameters {
    pub length: u32,
    pub byte_offset: u64,
    pub key: u32,
}

#[derive(Debug, Clone)]
pub struct WriteParameters {
    pub length: u32,
    pub byte_offset: u64,
    pub key: u32,
}

#[derive(Debug, Clone)]
pub struct DeviceControlParameters {
    pub ioctl_code: u32,
    pub input_buffer_length: u32,
    pub output_buffer_length: u32,
    pub method: u32,
}

#[derive(Debug, Clone)]
pub struct PnpParameters {
    pub minor_function: u32,
    pub system_context: u64,
}

#[derive(Debug, Clone)]
pub struct PowerParameters {
    pub system_power_state: u32,
    pub device_power_state: u32,
    pub wait: bool,
}

/// Driver Object - represents a loaded driver
#[derive(Debug)]
pub struct DriverObject {
    pub name: String,
    pub driver_extension: DriverExtension,
    pub major_functions: [Option<DriverDispatch>; 28],
    pub flags: DriverFlags,
    pub device_objects: Vec<Arc<DeviceObject>>,
}

#[derive(Debug)]
pub struct DriverExtension {
    pub add_device: Option<fn(&mut DriverObject, &str) -> NtStatus>,
    pub driver_unload: Option<fn(&mut DriverObject)>,
    pub dispatch_exceptions: Option<fn(&mut Irp) -> NtStatus>,
    pub key: String,
    pub service_parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub struct DriverFlags {
    pub unloading: bool,
    pub fs_driver: bool,
    pub filter_driver: bool,
}

/// Function pointer type for dispatch routines
pub type DriverDispatch = fn(&mut DeviceObject, &mut Irp) -> NtStatus;

/// Device Object - represents a device managed by a driver
#[derive(Debug)]
pub struct DeviceObject {
    pub name: String,
    pub device_type: DeviceType,
    pub characteristics: u32,
    pub reference_count: u32,
    pub driver_object: Arc<DriverObject>,
    pub next_device: Option<Arc<DeviceObject>>,
    pub attached_device: Option<Arc<DeviceObject>>,
    pub flags: DeviceFlags,
    pub device_extension: DeviceExtension,
    pub security_descriptor: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum DeviceType {
    FileSystem = 0x00000009,
    Keyboard = 0x0000000B,
    Mouse = 0x0000000D,
    SerialMousePort = 0x0000001A,
    SerialKeyboardPort = 0x0000001B,
    Disk = 0x00000007,
    Tape = 0x00000006,
    Network = 0x00000012,
    Screen = 0x00000001,
    Null = 0x00000000,
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceFlags {
    pub verified_access: bool,
    pub cdrom: bool,
    pub filesystem: bool,
    pub direct_io: bool,
    pub buffered_io: bool,
    pub exclusive: bool,
}

#[derive(Debug, Clone)]
pub struct DeviceExtension {
    pub extension_type: ExtensionType,
    pub data: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum ExtensionType {
    Disk,
    Network,
    FileSystem,
    Custom(String),
}

impl DriverObject {
    /// Create a new driver object
    pub fn new(name: &str) -> Self {
        // Initialize all major functions to None
        let major_functions: [Option<DriverDispatch>; 28] = std::array::from_fn(|_| None);
        
        Self {
            name: name.to_string(),
            driver_extension: DriverExtension {
                add_device: None,
                driver_unload: None,
                dispatch_exceptions: None,
                key: format!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\{}", name),
                service_parameters: HashMap::new(),
            },
            major_functions,
            flags: DriverFlags {
                unloading: false,
                fs_driver: false,
                filter_driver: false,
            },
            device_objects: Vec::new(),
        }
    }

    /// Register a dispatch routine for a major function
    pub fn register_dispatch(&mut self, major: MajorFunction, handler: DriverDispatch) {
        self.major_functions[major as usize] = Some(handler);
    }

    /// Process an IRP through the driver's dispatch table
    pub fn process_irp(&self, device: &mut DeviceObject, irp: &mut Irp) -> NtStatus {
        let major_idx = irp.major_function as usize;
        
        if major_idx >= self.major_functions.len() {
            return NtStatus::InvalidParameter;
        }

        match self.major_functions[major_idx] {
            Some(handler) => handler(device, irp),
            None => NtStatus::InvalidParameter,
        }
    }
}

impl DeviceObject {
    /// Create a new device object
    pub fn new(name: &str, device_type: DeviceType, driver: Arc<DriverObject>) -> Self {
        Self {
            name: name.to_string(),
            device_type,
            characteristics: 0,
            reference_count: 1,
            driver_object: driver,
            next_device: None,
            attached_device: None,
            flags: DeviceFlags {
                verified_access: false,
                cdrom: false,
                filesystem: false,
                direct_io: true,
                buffered_io: false,
                exclusive: false,
            },
            device_extension: DeviceExtension {
                extension_type: ExtensionType::Custom("default".to_string()),
                data: HashMap::new(),
            },
            security_descriptor: Vec::new(),
        }
    }

    /// Attach a device to the top of a device stack
    pub fn attach_device(&mut self, lower_device: Arc<DeviceObject>) {
        self.attached_device = Some(lower_device);
    }

    /// Reference the device object
    pub fn reference(&mut self) {
        self.reference_count += 1;
    }

    /// Dereference the device object
    pub fn dereference(&mut self) -> u32 {
        self.reference_count = self.reference_count.saturating_sub(1);
        self.reference_count
    }
}

// Common dispatch routines
pub fn default_create_handler(_device: &mut DeviceObject, _irp: &mut Irp) -> NtStatus {
    NtStatus::Success
}

pub fn default_close_handler(_device: &mut DeviceObject, _irp: &mut Irp) -> NtStatus {
    NtStatus::Success
}

pub fn default_read_handler(_device: &mut DeviceObject, _irp: &mut Irp) -> NtStatus {
    NtStatus::Pending
}

pub fn default_write_handler(_device: &mut DeviceObject, _irp: &mut Irp) -> NtStatus {
    NtStatus::Pending
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_creation() {
        let driver = DriverObject::new("TestDriver");
        assert_eq!(driver.name, "TestDriver");
        assert!(!driver.flags.unloading);
    }

    #[test]
    fn test_dispatch_registration() {
        let mut driver = DriverObject::new("TestDriver");
        driver.register_dispatch(MajorFunction::Create, default_create_handler);
        driver.register_dispatch(MajorFunction::Close, default_close_handler);
        
        assert!(driver.major_functions[MajorFunction::Create as usize].is_some());
        assert!(driver.major_functions[MajorFunction::Close as usize].is_some());
    }

    #[test]
    fn test_device_creation() {
        let driver = Arc::new(DriverObject::new("TestDriver"));
        let device = DeviceObject::new("\\Device\\TestDevice", DeviceType::Null, driver.clone());
        
        assert_eq!(device.name, "\\Device\\TestDevice");
        assert_eq!(device.device_type, DeviceType::Null);
        assert_eq!(device.reference_count, 1);
    }

    #[test]
    fn test_irp_processing() {
        let mut driver = DriverObject::new("TestDriver");
        driver.register_dispatch(MajorFunction::Create, default_create_handler);
        
        let mut device = DeviceObject::new(
            "\\Device\\TestDevice",
            DeviceType::Null,
            Arc::new(driver.clone())
        );
        
        let mut irp = Irp {
            major_function: MajorFunction::Create,
            minor_function: 0,
            io_status: IoStatusBlock {
                status: NtStatus::Success,
                information: 0,
            },
            associated_irp: None,
            user_buffer: Vec::new(),
            parameters: IrpParameters::Other,
            cancel: false,
            cancel_reason: None,
        };
        
        let result = driver.process_irp(&mut device, &mut irp);
        assert_eq!(result, NtStatus::Success);
    }

    #[test]
    fn test_device_reference_counting() {
        let driver = Arc::new(DriverObject::new("TestDriver"));
        let mut device = DeviceObject::new("\\Device\\Test", DeviceType::Null, driver);
        
        device.reference();
        device.reference();
        assert_eq!(device.reference_count, 3);
        
        device.dereference();
        device.dereference();
        assert_eq!(device.reference_count, 1);
    }
}
