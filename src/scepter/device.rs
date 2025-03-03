//! Basic routines to initialize or shut down a device and to set/get parameters.

use std::os::raw::c_char;
use std::{ffi::CStr, time::Duration};
use sys::ScStatus_SC_OK as OK;

use vzense_sys::scepter as sys;

use crate::{ColorFormat, ColorResolution, Resolution, cyan, red, yellow};

use super::get_message;

/// The main interface to the camera.
pub struct Device {
    pub(super) handle: sys::ScDeviceHandle,
    pub(super) frame_ready: sys::ScFrameReady,
    pub(super) frame: sys::ScFrame,
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
                "{}",
                cyan!("model: {}, IP: {}, firmware: {}", info[0], info[1], info[2])
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
            println!("{}", red!("!!! pixel count is not equal to {} * {}", w, h))
        }
    }

    /// Set the color frame format to either RGB or BGR.
    pub fn set_color_format(&self, format: ColorFormat) {
        unsafe {
            match format {
                ColorFormat::Rgb => sys::scSetColorPixelFormat(
                    self.handle,
                    sys::ScPixelFormat_SC_PIXEL_FORMAT_RGB_888,
                ),
                ColorFormat::Bgr => sys::scSetColorPixelFormat(
                    self.handle,
                    sys::ScPixelFormat_SC_PIXEL_FORMAT_BGR_888,
                ),
            };
        }
    }

    /// Enable or disable the mapping of the color image to depth camera space.
    pub fn map_color_to_depth(&mut self, is_enabled: bool) {
        if self.color_resolution != ColorResolution::Res640x480 {
            self.set_color_resolution(ColorResolution::Res640x480);
            self.color_resolution = ColorResolution::Res640x480;
        }
        unsafe {
            sys::scSetTransformColorImgToDepthSensorEnabled(
                self.handle,
                if is_enabled { 1 } else { 0 },
            );
        }
        self.color_is_mapped = is_enabled;
    }

    /// Sets the resolution of the color frame and also returns it. Three resolutions are currently available: 640x480, 800x600, and 1600x1200.
    pub fn set_color_resolution(&mut self, resolution: ColorResolution) -> Resolution {
        if self.color_is_mapped {
            println!(
                "{}",
                yellow!(
                    "setting of color resolution is ignored because color frame is mapped to depth"
                )
            );
        } else {
            unsafe {
                match resolution {
                    ColorResolution::Res640x480 => sys::scSetColorResolution(self.handle, 640, 480),
                    ColorResolution::Res800x600 => sys::scSetColorResolution(self.handle, 800, 600),
                    ColorResolution::Res1600x1200 => {
                        sys::scSetColorResolution(self.handle, 1600, 1200)
                    }
                };
            }
            self.color_resolution = resolution;
        }
        self.get_color_resolution()
    }

    /// Returns the resolution of the color frame.
    pub fn get_color_resolution(&self) -> Resolution {
        let mut w = 0;
        let mut h = 0;
        unsafe {
            sys::scGetColorResolution(self.handle, &mut w, &mut h);
        }
        Resolution::new(w as u32, h as u32)
    }

    /// Current work mode.
    pub fn get_work_mode(&self) -> Result<u32, String> {
        let mut work_mode = sys::ScWorkMode::default();
        let status = unsafe { sys::scGetWorkMode(self.handle, &mut work_mode) };
        if status != OK {
            return Err(red!("get work mode failed with status {}", status));
        }
        Ok(work_mode)
    }

    /// Stops the stream, closes the device, and clears all resources.
    pub fn shut_down(&mut self, verbose: bool) {
        unsafe {
            sys::scStopStream(self.handle);
            sys::scCloseDevice(&mut self.handle);

            let status = sys::scShutdown();
            if status != OK {
                println!(
                    "{}",
                    red!("shut down failed with status: {}", get_message(status))
                );
            } else if verbose {
                println!("shut down device successfully");
            }
        }
    }

    /// Returns device info as an array of Strings: \[model, IP, firmware, serial number\]
    pub fn get_device_info(&self, device_count: u32) -> Result<[String; 4], String> {
        if device_count == 0 {
            return Err(red!("no device to get info for"));
        }

        let mut device_info = sys::ScDeviceInfo::default();

        unsafe { sys::scGetDeviceInfoList(device_count, &mut device_info) };

        let ip = device_info.ip.as_ptr();
        let model = device_info.productName.as_ptr();
        let serial = device_info.serialNumber.as_ptr();

        let firmware = self
            .get_firmware_version()
            .expect(&red!("cannot get firmware version"));

        Ok([
            unsafe { CStr::from_ptr(model) }
                .to_string_lossy()
                .into_owned(),
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
        let mut handle = 0 as sys::ScDeviceHandle;
        let status = unsafe { sys::scOpenDeviceByIP(ip, &mut handle) };
        if status != OK {
            return Err(red!(
                "open device failed with status {}",
                get_message(status)
            ));
        }
        if !handle.is_null() {
            Ok(Device {
                handle,
                frame_ready: sys::ScFrameReady::default(),
                frame: sys::ScFrame::default(),
                color_resolution: ColorResolution::Res640x480,
                color_is_mapped: false,
                current_frame_is_depth: false,
                min_depth_mm: 500,  // default value
                max_depth_mm: 1000, // default value
            })
        } else {
            Err(red!("device ptr is null"))
        }
    }

    fn start_stream(&self, verbose: bool) -> Result<(), String> {
        let status = unsafe { sys::scStartStream(self.handle) };
        if status != OK {
            return Err(red!("start stream failed with status {}", status));
        }
        if verbose {
            println!("stream started")
        }
        Ok(())
    }

    /// Returns the current depth measuring range `(min, max)` of the camera in mm. **Note**: At least the min value seems to have no practical meaning. For the NYX650 the returned min value is 1 mm which makes no sense, while the max value is 4700 mm. In the specs the depth range for the NYX650 is given as min: 300 mm, max: 4500 mm.
    #[deprecated]
    pub fn get_depth_measuring_range(&self) -> (u16, u16) {
        unsafe {
            let mut min = 0;
            let mut max = 0;
            sys::scGetDepthRangeValue(self.handle, &mut min, &mut max);
            (min as u16, max as u16)
        }
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

fn get_firmware_version(handle: sys::ScDeviceHandle, buffer: &mut [u8]) -> sys::ScStatus {
    let len = buffer.len().try_into().unwrap();
    let ptr: *mut c_char = buffer.as_mut_ptr().cast();
    unsafe { sys::scGetFirmwareVersion(handle, ptr, len) }
}

fn initialize(verbose: bool) -> Result<(), String> {
    if verbose {
        println!("initializing...");
    }
    let status = unsafe { sys::scInitialize() };

    // status -101 is reinitialization
    if status == -101 {
        if verbose {
            println!("reinitializing...");
        }
    } else if status != OK {
        return Err(red!("initialization failed with status {}", status));
    }
    Ok(())
}

/// Tries to find devices every 200 ms for duration `scan_time`.
fn get_device_count(scan_time: Duration, verbose: bool) -> Result<u32, String> {
    if scan_time < Duration::from_secs(1) {
        println!(
            "{}",
            yellow!("vzense-rust warning: scan time might be too short to detect a device")
        );
    }
    let scan_interval = Duration::from_millis(200);
    let try_count = scan_time.div_duration_f64(scan_interval).ceil() as u64;

    let mut device_count = 0;
    let mut times_tried = 0;
    let mut status;
    if verbose {
        println!("searching for device...");
    }
    loop {
        status = unsafe { sys::scGetDeviceCount(&mut device_count, 200) };

        if status != OK {
            return Err(red!("get device count failed with status {}", status));
        } else {
            if device_count > 0 {
                if verbose {
                    println!("{}", cyan!("device found"));
                }
                break;
            }
            times_tried += 1;
            if times_tried >= try_count {
                return Err(red!("no device found"));
            }
        }
    }
    Ok(device_count)
}

fn get_ip(device_count: u32) -> Result<*const c_char, String> {
    let mut device_info = sys::ScDeviceInfo::default();
    unsafe {
        let status = sys::scGetDeviceInfoList(device_count, &mut device_info);
        if status != OK {
            return Err(red!("get device list info failed with status {}", status));
        }
    }
    Ok(device_info.ip.as_ptr())
}
