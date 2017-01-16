use libc;
use libc::c_void;
use std::{mem, result, ptr, fmt, ffi};
use std::iter::Iterator;
use tv;

const CEC_MAX_DATA_PACKET_SIZE: usize = 16 * 4;
const CEC_VERSION_MAJOR: u32 = 3;
const CEC_VERSION_MINOR: u32 = 0;
const CEC_VERSION_PATCH: u32 = 1;
// Reporting versions before 2.3 is not supported...
const CEC_VERSION_CURRENT: u32 = (CEC_VERSION_MAJOR << 16) | (CEC_VERSION_MINOR << 8) | CEC_VERSION_PATCH;

type LibcecConnectionT = *mut c_void;

pub type Result<T> = result::Result<T, CecError>;

#[derive(Debug)]
pub enum CecError {
    InitFailed,
    FindAdaptersFailed,
    NoAdapterFound,
    OpenFailed
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
    message: *const libc::c_char,
    level: CecLogLevel,
    time: i64
}

#[repr(C)]
#[allow(dead_code)]
enum CecUserControlCode {
    SELECT = 0x00
    // TODO: Add morehttps://github.com/Pulse-Eight/libcec
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
    log_message: extern fn(*mut c_void, *const CecLogMessage) -> libc::c_int,
    key_press: extern fn(*mut c_void, *const CecKeypress) -> libc::c_int,
    command_received: extern fn(*mut c_void, *const CecCommand) -> libc::c_int,
    configuration_changed: extern fn(*mut c_void, *const LibcecConfiguration) -> libc::c_int,
    alert: extern fn(*mut c_void, LibcecAlert, LibcecParameter) -> libc::c_int,
    menu_state_changed: extern fn(*mut c_void, CecMenuState) -> libc::c_int,
    source_activated: extern fn(*mut c_void, CecLogicalAddress, u8)
}

static mut ICECCALLBACKS_DEFAULT: ICECCallbacks = ICECCallbacks {
    log_message: ICECCallbacks::default_log_message,
    key_press: ICECCallbacks::default_key_press,
    command_received: ICECCallbacks::default_command_received,
    configuration_changed: ICECCallbacks::default_configuration_changed,
    alert: ICECCallbacks::default_alert,
    menu_state_changed: ICECCallbacks::default_menu_state_changed,
    source_activated: ICECCallbacks::default_source_activated
};

impl ICECCallbacks {
    extern fn default_log_message(cb_param: *mut c_void, message: *const CecLogMessage) -> libc::c_int {
        println!("{:?}", message);
        1
    }

    extern fn default_key_press(cb_param: *mut c_void, key_press: *const CecKeypress) -> libc::c_int { 0 }
    extern fn default_command_received(cb_param: *mut c_void, command: *const CecCommand) -> libc::c_int { 0 }
    extern fn default_configuration_changed(cb_param: *mut c_void, config: *const LibcecConfiguration) -> libc::c_int { 0 }
    extern fn default_alert(cb_param: *mut c_void, alert: LibcecAlert, param: LibcecParameter) -> libc::c_int { 0 }
    extern fn default_menu_state_changed(cb_param: *mut c_void, menu_state: CecMenuState) -> libc::c_int { 0 }
    extern fn default_source_activated(cb_param: *mut c_void, address: CecLogicalAddress, x: u8) {}
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
    str_device_name: [libc::c_char; 13],
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
    str_device_language: [libc::c_char; 3],
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

#[repr(C)]
#[derive(Copy)]
struct CecAdapter {
    path: [libc::c_char; 1024],
    comm: [libc::c_char; 1024]
}

impl CecAdapter {
    fn empty() -> CecAdapter {
        CecAdapter {
            path: [0; 1024],
            comm: [0; 1024]
        }
    }
}

impl Clone for CecAdapter {
    fn clone(&self) -> CecAdapter {
        CecAdapter {
            path: self.path,
            comm: self.comm
        }
    }
}

impl fmt::Debug for CecAdapter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            fmt.debug_struct("CecAdapter")
                .field("path", &ffi::CStr::from_ptr(self.path.as_ptr()))
                .field("comm", &ffi::CStr::from_ptr(self.comm.as_ptr()))
                .finish()
        }
    }
}

#[link(name = "cec")]
extern {
    fn libcec_initialise(configuration: *mut LibcecConfiguration) -> LibcecConnectionT;
    fn libcec_destroy(connection: LibcecConnectionT);
    fn libcec_open(connection: LibcecConnectionT, str_port: *const libc::c_char, i_timeout: u32) -> libc::c_int;
    fn libcec_close(connection: LibcecConnectionT);

    fn libcec_init_video_standalone(connection: LibcecConnectionT);
    fn libcec_find_adapters(connection: LibcecConnectionT, device_list: *mut CecAdapter, i_buf_size: u8, str_device_path: *mut libc::c_char) -> i8;

    fn libcec_power_on_devices(connection: LibcecConnectionT, cec_logical_address: CecLogicalAddress) -> libc::c_int;
    fn libcec_standby_devices(connection: LibcecConnectionT, cec_logical_address: CecLogicalAddress) -> libc::c_int;
    fn libcec_set_inactive_view(connection: LibcecConnectionT) -> libc::c_int;

    fn libcec_clear_configuration(configuration: *mut LibcecConfiguration);
}

impl LibcecConfiguration {
    fn new(callbacks: &'static mut ICECCallbacks) -> LibcecConfiguration {
        unsafe {
            let mut config = mem::zeroed::<LibcecConfiguration>();
            libcec_clear_configuration(&mut config);
            config.client_version = CEC_VERSION_CURRENT;
            config.b_activate_source = 0;
            config.device_types.types[0] = CecDeviceType::RECORDING_DEVICE;
            config.callbacks = callbacks;
            config
        }
    }
}

pub struct Connection {
    conn: LibcecConnectionT,
    #[allow(dead_code)]
    config: LibcecConfiguration
}

impl Connection {
    pub fn new() -> Result<Connection> {
        unsafe {
            let mut config = LibcecConfiguration::new(&mut ICECCALLBACKS_DEFAULT);
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
        let adapters = self.find_adapters()?;
        let adapter = adapters.first().ok_or(CecError::NoAdapterFound)?;
        println!("Connecting to {:?}", adapter);
        unsafe {
            if libcec_open(self.conn, adapter.comm.as_ptr(), 5000) == 0 {
                return Err(CecError::OpenFailed)
            }
            libcec_set_inactive_view(self.conn);
        }
        Ok(())
    }

    fn find_adapters(&mut self) -> Result<Vec<CecAdapter>> {
        let mut buf = [CecAdapter::empty(); 10];
        let adapter_count = unsafe {
          libcec_find_adapters(self.conn, buf.as_mut_ptr(), buf.len() as u8, ptr::null_mut())
        };
        if adapter_count >= 0 {
            Ok(buf.into_iter().take(adapter_count as usize).map(|x| *x).collect())
        } else {
            Err(CecError::FindAdaptersFailed)
        }
    }
}

impl tv::TVController for Connection {
    fn turn_on_tv(&mut self) -> tv::Result<()> {
        unsafe {
            if libcec_power_on_devices(self.conn, CecLogicalAddress::TV) == 0 {
                println!("Huh, unable to turn on the TV...");
                Err(tv::TVError::TVControlFailed)
            } else {
                Ok(())
            }
        }
    }

    fn turn_off_tv(&mut self) -> tv::Result<()> {
        unsafe {
            if libcec_standby_devices(self.conn, CecLogicalAddress::TV) == 0 {
                println!("Huh, unable to turn off the TV...");
                Err(tv::TVError::TVControlFailed)
            } else {
                Ok(())
            }
        }
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
