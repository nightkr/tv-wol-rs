use libc;
use libc::c_void;
use std::{mem, result};

const CEC_MAX_DATA_PACKET_SIZE: usize = 16 * 4;
const CEC_VERSION_MAJOR: u32 = 3;
const CEC_VERSION_MINOR: u32 = 0;
const CEC_VERSION_PATCH: u32 = 1;
// Reporting versions before 2.3 is not supported...
const CEC_VERSION_CURRENT: u32 = (CEC_VERSION_MAJOR << 16) | (CEC_VERSION_MINOR << 8) | CEC_VERSION_PATCH;

type LibcecConnectionT = *mut c_void;

type Result<T> = result::Result<T, CecError>;

#[derive(Debug)]
pub enum CecError {
    InitFailed
}

#[repr(C)]
#[allow(dead_code)]
enum CecDeviceType {
    TV = 0,
    RECORDING_DEVICE = 1,
    RESERVED = 2,
    TUNER = 3,
    PLAYBACK_DEVICE = 4,
    AUDIO_SYSTEM = 5
}

#[repr(C)]
#[allow(dead_code)]
struct CecDeviceTypeList {
    types: [CecDeviceType; 5]
}

#[repr(C)]
#[allow(dead_code)]
enum CecMenuState {
    ACTIVATED = 0,
    DEACTIVATED = 1
}

#[repr(C)]
#[allow(dead_code)]
enum CecLogLevel {
    ERROR = 1,
    WARNING = 2,
    NOTICE = 4,
    TRAFFIC = 8,
    DEBUG = 16,
    ALL = 31
}

#[repr(C)]
struct CecLogMessage {
    message: *const libc::wchar_t,
    level: CecLogLevel,
    time: i64
}

#[repr(C)]
#[allow(dead_code)]
enum CecUserControlCode {
    SELECT = 0x00
    // TODO: Add more
}

#[repr(C)]
struct CecKeypress {
    keycode: CecUserControlCode,
    duration: libc::c_int
}

#[repr(C)]
#[allow(dead_code)]
enum CecLogicalAddress {
    UNKNOWN = -1,
    TV = 0
    // TODO: add more
}

#[repr(C)]
struct CecLogicalAddresses {
    primary: CecLogicalAddress,
    addresses: [libc::c_int; 16]
}

#[repr(C)]
#[allow(dead_code)]
enum CecOpcode {
    ACTIVATE_SOURCE = 0x82
    // TODO: Add more
}

#[repr(C)]
#[allow(dead_code)]
struct CecDatapacket {
    data: [u8; CEC_MAX_DATA_PACKET_SIZE],
    size: u8
}

#[repr(C)]
#[allow(dead_code)]
enum LibcecAlert {
    SERVICE_DEVICE,
    CONNECTION_LOST,
    PERMISSION_ERROR,
    PORT_BUSY,
    PHYSICAL_ADDRESS_ERROR,
    TV_POLL_FAILED
}

#[repr(C)]
#[allow(dead_code)]
enum LibcecParameterType {
    STRING,
    UNKNOWN
}

#[repr(C)]
struct LibcecParameter {
    param_type: LibcecParameterType,
    param_data: *mut c_void
}

#[repr(C)]
struct CecCommand {
    initiator: CecLogicalAddress,
    destination: CecLogicalAddress,
    ack: i8,
    eom: i8,
    opcode: CecOpcode,
    parameters: CecDatapacket,
    opcode_set: i8,
    transmit_timeout: i32
}

#[repr(C)]
struct ICECCallbacks {
    log_message: extern fn(*mut c_void, *const CecLogMessage),
    key_press: extern fn(*mut c_void, *const CecKeypress),
    command_received: extern fn(*mut c_void, *const CecCommand),
    configuration_changed: extern fn(*mut c_void, *const LibcecConfiguration),
    alert: extern fn(*mut c_void, LibcecAlert, LibcecParameter),
    menu_state_changed: extern fn(*mut c_void, CecMenuState),
    source_activated: extern fn(*mut c_void, CecLogicalAddress, u8)
}

#[repr(C)]
#[allow(dead_code)]
enum CecVersion {
    UNKNOWN = 0x00,
    V_1_2 = 0x01,
    V_1_2A = 0x02,
    V_1_3 = 0x03,
    V_1_3A = 0x04,
    V_1_4 = 0x05
}

#[repr(C)]
#[allow(dead_code)]
enum CecAdapterType {
    UNKNOWN = 0,
    P8_EXTERNAL = 0x1,
    P8_DAUGHTERBOARD = 0x2,
    RPI = 0x100,
    TDA995x = 0x200,
    EXYNOS = 0x300,
    AOCEC = 0x500
}

#[repr(C)]
#[allow(dead_code)]
struct LibcecConfiguration {
    client_version: u32,
    str_device_name: [libc::wchar_t; 13],
    device_types: CecDeviceTypeList,
    b_autodetect_address: u8,
    i_physical_address: u16,
    base_device: CecLogicalAddress,
    i_hdmi_port: u8,
    tv_vendor: u32,
    wake_devices: CecLogicalAddresses,
    power_off_devices: CecLogicalAddresses,
    server_version: u32,
    b_get_settings_from_rom: u8,
    b_activate_source: u8,
    b_power_off_on_standby: u8,
    callback_param: *mut c_void,
    callbacks: *mut ICECCallbacks,
    logical_addresses: CecLogicalAddresses,
    i_firmware_version: u16,
    str_device_language: [libc::wchar_t; 3],
    i_firmware_build_date: u32,
    b_monitor_only: u8,
    cec_version: CecVersion,
    adapter_type: CecAdapterType,
    combo_key: CecUserControlCode,
    i_combo_key_timeout_ms: u32,
    i_button_repeat_rate_ms: u32,
    i_button_release_delay_ms: u32,
    i_double_tap_timeout_ms: u32,
    b_auto_wake_avr: u8
}

#[link(name = "cec")]
extern {
    fn libcec_initialise(configuration: *mut LibcecConfiguration) -> LibcecConnectionT;
    fn libcec_destroy(connection: LibcecConnectionT);
    fn libcec_close(connection: LibcecConnectionT);

    fn libcec_init_video_standalone(connection: LibcecConnectionT);

    fn libcec_clear_configuration(configuration: *mut LibcecConfiguration);
}

impl LibcecConfiguration {
    fn new() -> LibcecConfiguration {
        unsafe {
            let mut config = mem::zeroed::<LibcecConfiguration>();
            libcec_clear_configuration(&mut config);
            config.client_version = CEC_VERSION_CURRENT;
            config
        }
    }
}

pub struct Connection {
    conn: LibcecConnectionT,
    config: LibcecConfiguration
}

impl Connection {
    pub fn new() -> Result<Connection> {
        let mut config = LibcecConfiguration::new();
        unsafe {
            let conn = libcec_initialise(&mut config);
            if conn as usize == 0 {
                Err(CecError::InitFailed)
            } else {
                Ok(Connection {
                    conn: conn,
                    config: config
                })
            }
        }
    }

    pub fn init(&mut self) -> Result<()> {
        unsafe {
            libcec_init_video_standalone(self.conn);
        }
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            libcec_close(self.conn);
            libcec_destroy(self.conn);
        }
    }
}
