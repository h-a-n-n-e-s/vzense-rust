//! Basic routines to initialize or shut down a device and to set/get parameters.

use std::{ffi::CStr, os::raw::c_char, thread::sleep, time::Duration};

use sys::PsReturnStatus_PsRetOK as OK;
use vzense_sys::dcam560 as sys;

use crate::{ColorFormat, ColorResolution, DepthMeasuringRange, Resolution};

use super::SESSION_INDEX;

/// A wrapper for the raw pointer `handle` used in every `vzense_sys` call. It also includes `frame_ready` (containing frame availability data) and `frame` (containing a pointer to the actual frame data).
pub struct Device {
    pub(super) handle: sys::PsDeviceHandle,
    pub(super) frame_ready: sys::PsFrameReady,
    pub(super) frame: sys::PsFrame,
    pub(super) color_resolution: ColorResolution,
    pub(super) color_is_mapped: bool,
    pub(super) current_frame_is_depth: bool,
    pub(super) min_depth_mm: u16,
    pub(super) max_depth_mm: u16,
}
impl Device {
    /// Initializes the sytem and returns a device if it finds one. Make sure a Vzense camera is connected. `scan_time` should be at least one second to find a device. Set `scan_time = Duration::MAX` to scan until a device was found (useful to wait for reconnection after the connection to a device was interrupted).
    pub fn initialize(scan_time: Duration, verbose: bool) -> Result<Self, String> {
        initialize(verbose)?;

        let device_count = get_device_count(scan_time, verbose)?;

        let mut device = Device::open_device_by_ip(get_ip(device_count)?)?;

        if verbose {
            let info = device.get_device_info(device_count)?;
            println!(
                "\x1b[36mmodel: {}, IP: {}, firmware: {}\x1b[0m",
                info[0], info[1], info[2]
            );
        }

        device.start_stream(verbose)?;

        device.set_color_resolution(ColorResolution::Res640x480);

        Ok(device)
    }

    /// Choosing the min/max depth in mm for the color mapping of the depth output. These values also bound the depths used in the `util::TochDetector` to reduce measuring artifacts.
    pub fn set_depth_range(&mut self, min_depth_mm: u16, max_depth_mm: u16) {
        self.min_depth_mm = min_depth_mm;
        self.max_depth_mm = max_depth_mm;
    }

    /// Get frame info like frame type, pixel format, width, height, etc.
    pub fn get_frame_info(&self) -> String {
        format!("{:?}", self.frame)
    }

    /// Checks if the number of pixels in `frame` equals `pixel_count`.
    pub fn check_pixel_count(&self, pixel_count: usize) {
        let w = self.frame.width as usize;
        let h = self.frame.height as usize;
        if w * h != pixel_count {
            println!(
                "\x1b[31m!!! pixel count is not equal to {} * {}\x1b[0m",
                w, h
            )
        }
    }

    /// Set the color frame format to either RGB or BGR.
    pub fn set_color_format(&self, format: ColorFormat) {
        unsafe {
            match format {
                ColorFormat::Rgb => sys::Ps2_SetColorPixelFormat(
                    self.handle,
                    SESSION_INDEX,
                    sys::PsPixelFormat_PsPixelFormatRGB888,
                ),
                ColorFormat::Bgr => sys::Ps2_SetColorPixelFormat(
                    self.handle,
                    SESSION_INDEX,
                    sys::PsPixelFormat_PsPixelFormatBGR888,
                ),
            };
        }
    }

    /// Enable or disable the mapping of the color image to depth camera space.
    pub fn map_color_to_depth(&mut self, is_enabled: bool) {
        if self.color_resolution != ColorResolution::Res640x480 {
            self.set_color_resolution(ColorResolution::Res640x480);
        }
        unsafe {
            // should actually be `Ps2_SetMapperEnabledRGBToDepth` but the names seem to be mixed up
            sys::Ps2_SetMapperEnabledDepthToRGB(
                self.handle,
                SESSION_INDEX,
                if is_enabled { 1 } else { 0 },
            );
        }
        self.color_is_mapped = is_enabled;
    }

    /// Sets the resolution of the color frame. Three resolutions are currently available: 640x480, 800x600, and 1600x1200.
    pub fn set_color_resolution(&mut self, resolution: ColorResolution) -> Resolution {
        if self.color_is_mapped {
            println!(
                "\x1b[33msetting of color resolution is ignored because color frame is mapped to depth\x1b[0m"
            );
        } else {
            let res = match resolution {
                ColorResolution::Res640x480 => sys::PsResolution_PsRGB_Resolution_640_480,
                ColorResolution::Res800x600 => sys::PsResolution_PsRGB_Resolution_800_600,
                ColorResolution::Res1600x1200 => sys::PsResolution_PsRGB_Resolution_1600_1200,
            };
            unsafe {
                sys::Ps2_SetRGBResolution(self.handle, SESSION_INDEX, res);
            }
            self.color_resolution = resolution;
        }
        self.get_color_resolution()
    }

    /// Returns the resolution of the color frame.
    pub fn get_color_resolution(&self) -> Resolution {
        let mut resolution_type = 0;
        unsafe {
            sys::Ps2_GetRGBResolution(self.handle, SESSION_INDEX, &mut resolution_type);
        }
        match resolution_type {
            2 => Resolution::new(640, 480),
            5 => Resolution::new(800, 600),
            4 => Resolution::new(1600, 1200),
            _ => panic!("\x1b[31munknown rgb resolution\x1b[0m"),
        }
    }

    /// Sets the depth range mode.
    pub fn set_depth_measuring_range(&self, depth_range: DepthMeasuringRange) {
        let depth_range = match depth_range {
            DepthMeasuringRange::Near => 0,
            DepthMeasuringRange::Mid => 1,
            DepthMeasuringRange::Far => 2,
        };
        unsafe {
            sys::Ps2_SetDepthRange(self.handle, SESSION_INDEX, depth_range);
        }
    }

    /// Returns the current depth measuring range `(min, max)` of the camera in mm.
    pub fn get_depth_measuring_range(&self) -> (u16, u16) {
        let mut depth_range = sys::PsDepthRange::default();

        unsafe {
            sys::Ps2_GetDepthRange(self.handle, SESSION_INDEX, &mut depth_range);
        }

        let mut mr = sys::PsMeasuringRange::default();

        unsafe {
            sys::Ps2_GetMeasuringRange(self.handle, SESSION_INDEX, depth_range, &mut mr);
        }

        match depth_range {
            0 => (mr.effectDepthMinNear, mr.effectDepthMaxNear),
            1 => (mr.effectDepthMinMid, mr.effectDepthMaxMid),
            2 => (mr.effectDepthMinFar, mr.effectDepthMaxFar),
            _ => panic!("\x1b[31munknown measuring range\x1b[0m"),
        }
    }

    /// Set the wait time for the call to `read_next_frame` in ms.
    pub fn set_wait_time(&self, time: u16) {
        unsafe {
            sys::Ps2_SetWaitTimeOfReadNextFrame(self.handle, SESSION_INDEX, time);
        }
    }

    /// Current data mode.
    pub fn get_data_mode(&self) -> Result<u32, String> {
        let mut data_mode = sys::PsDataMode::default();
        let status = unsafe { sys::Ps2_GetDataMode(self.handle, SESSION_INDEX, &mut data_mode) };
        if status != OK {
            return Err(format!(
                "\x1b[31mget data mode failed with status {}\x1b[0m",
                status
            ));
        }
        Ok(data_mode)
    }

    /// Stops the stream, closes the device, and clears all resources.
    pub fn shut_down(&mut self, verbose: bool) {
        unsafe {
            sys::Ps2_StopStream(self.handle, SESSION_INDEX);
            sys::Ps2_CloseDevice(&mut self.handle);

            let status = sys::Ps2_Shutdown();
            if status != OK {
                println!("\x1b[31mshut down failed with status: {}\x1b[0m", status);
            } else if verbose {
                println!("shut down device successfully");
            }
        }
    }

    /// Returns device info as an array of Strings: \[model, IP, firmware, serial number\]
    pub fn get_device_info(&self, device_count: u32) -> Result<[String; 4], String> {
        if device_count == 0 {
            return Err("\x1b[31mno device to get info for\x1b[0m".to_string());
        }

        let mut device_info = sys::PsDeviceInfo::default();

        unsafe { sys::Ps2_GetDeviceListInfo(&mut device_info, device_count) };

        let ip = device_info.ip.as_ptr();
        let uri = device_info.uri.as_ptr(); // model_name:serial_number
        let serial = device_info.alias.as_ptr(); // serial number

        let firmware = self
            .get_firmware_version()
            .expect("\x1b[31mCannot get firmware version\x1b[0m");

        Ok([
            unsafe { CStr::from_ptr(uri) }
                .to_str()
                .unwrap()
                .split(":")
                .collect::<Vec<&str>>()[0]
                .to_string(),
            unsafe { CStr::from_ptr(ip) }.to_string_lossy().into_owned(),
            firmware,
            unsafe { CStr::from_ptr(serial) }
                .to_string_lossy()
                .into_owned(),
        ])
    }

    // private functions_______________________________________________________

    fn get_firmware_version(&self) -> Result<String, String> {
        let mut buffer = [0; 64];
        match get_firmware_version(self.handle, &mut buffer) {
            OK => Ok(CStr::from_bytes_until_nul(&buffer)
                .unwrap()
                .to_string_lossy()
                .into_owned()),
            error_code => Err(format!("{}", error_code)),
        }
    }

    fn open_device_by_ip(ip: *const c_char) -> Result<Self, String> {
        let mut handle = 0 as sys::PsDeviceHandle;
        let status = unsafe { sys::Ps2_OpenDeviceByIP(ip, &mut handle) };
        if status != OK {
            return Err(format!(
                "\x1b[31mopen device failed with status {}\x1b[0m",
                status
            ));
        }
        if !handle.is_null() {
            Ok(Device {
                handle,
                frame_ready: sys::PsFrameReady::default(),
                frame: sys::PsFrame::default(),
                color_resolution: ColorResolution::Res640x480,
                color_is_mapped: false,
                current_frame_is_depth: false,
                min_depth_mm: 500,  // default value
                max_depth_mm: 1000, // default value
            })
        } else {
            Err("\x1b[31mdevice ptr is null\x1b[0m".to_string())
        }
    }

    fn start_stream(&self, verbose: bool) -> Result<(), String> {
        let status = unsafe { sys::Ps2_StartStream(self.handle, SESSION_INDEX) };
        if status != OK {
            return Err(format!(
                "\x1b[31mstart stream failed with status {}\x1b[0m",
                status
            ));
        }
        if verbose {
            println!("stream started")
        }
        Ok(())
    }

    #[deprecated(since = "0.3.0", note = "Use initialize(scan_time, verbose) instead.")]
    pub fn init() -> Result<Self, String> {
        Err("Deprecated, use initialize(scan_time, verbose) instead".to_string())
    }
}

/// `Data` trait to allow use of `Device` in `util::TouchDetector`.
impl crate::util::touch_detector::Data for Device {
    fn get_frame_p_frame_data(&self) -> *mut u8 {
        self.frame.pFrameData
    }
    fn get_frame_data_len(&self) -> usize {
        self.frame.dataLen as usize
    }
    fn get_min_depth_mm(&self) -> u16 {
        self.min_depth_mm
    }
    fn get_max_depth_mm(&self) -> u16 {
        self.max_depth_mm
    }
    fn current_frame_is_depth(&self) -> bool {
        self.current_frame_is_depth
    }
}

fn get_firmware_version(handle: sys::PsDeviceHandle, buffer: &mut [u8]) -> sys::PsReturnStatus {
    let len = buffer.len().try_into().unwrap();
    let ptr: *mut c_char = buffer.as_mut_ptr().cast();
    unsafe { sys::Ps2_GetFirmwareVersionNumber(handle, SESSION_INDEX, ptr, len) }
}

fn initialize(verbose: bool) -> Result<(), String> {
    if verbose {
        println!("initializing...");
    }
    let status = unsafe { sys::Ps2_Initialize() };

    // status -101 is reinitialization
    if status == -101 {
        if verbose {
            println!("reinitializing...");
        }
    } else if status != OK {
        return Err(format!(
            "\x1b[31minitialization failed with status {}\x1b[0m",
            status
        ));
    }
    Ok(())
}

/// Tries to find devices every 200 ms for duration `scan_time`.
fn get_device_count(scan_time: Duration, verbose: bool) -> Result<u32, String> {
    if scan_time < Duration::from_secs(1) {
        println!(
            "\x1b[33mvzense-rust warning: scan time might be too short to detect a device\x1b[0m"
        );
    }
    let sleep_interval = Duration::from_millis(200);
    let try_count = scan_time.div_duration_f64(sleep_interval).ceil() as u64;

    let mut device_count = 0;
    let mut times_tried = 0;
    let mut status;
    if verbose {
        println!("searching for device...");
    }
    loop {
        status = unsafe { sys::Ps2_GetDeviceCount(&mut device_count) };

        if status != OK {
            return Err(format!(
                "\x1b[31mget device count failed with status {}\x1b[0m",
                status
            ));
        } else {
            if device_count > 0 {
                if verbose {
                    println!("\x1b[36mdevice found, \x1b[0m");
                }
                break;
            }
            times_tried += 1;
            if times_tried >= try_count {
                return Err("\x1b[31mno device found\x1b[0m".to_string());
            }
            sleep(sleep_interval);
        }
    }
    Ok(device_count)
}

fn get_ip(device_count: u32) -> Result<*const c_char, String> {
    let mut device_info = sys::PsDeviceInfo::default();
    unsafe {
        let status = sys::Ps2_GetDeviceListInfo(&mut device_info, device_count);
        if status != OK {
            return Err(format!(
                "\x1b[31mget device list info failed with status {}\x1b[0m",
                status
            ));
        }
    }
    Ok(device_info.ip.as_ptr())
}
