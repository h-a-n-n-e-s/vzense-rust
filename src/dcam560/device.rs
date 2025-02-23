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
    /// Initializes the sytem and returns a device if it finds one. Make sure a Vzense camera is connected. After 3 seconds the routine will time out if no device was found.
    pub fn init() -> Result<Self, String> {
        unsafe {
            println!("initializing...");

            let mut status = sys::Ps2_Initialize();
            if status != OK {
                return Err(format!("initialization failed with status {}", status));
            }

            let mut device_count = 0;
            let mut times_tried = 0;
            println!("searching for device...");
            loop {
                status = sys::Ps2_GetDeviceCount(&mut device_count);
                if status != OK {
                    return Err(format!("get device count failed with status {}", status));
                } else {
                    if device_count > 0 {
                        print!("device found, ");
                        break;
                    }
                    times_tried += 1;
                    // give up after 3 seconds
                    if times_tried == 15 {
                        return Err("no device found".to_string());
                    }
                    sleep(Duration::from_millis(200));
                }
            }

            let mut device_info = sys::PsDeviceInfo::default();

            sys::Ps2_GetDeviceListInfo(&mut device_info, device_count);
            let ip: *const c_char = device_info.ip.as_ptr();
            let uri = device_info.uri.as_ptr(); // model_name:serial_number
            // let alias = device_info.alias; // serial number

            let device = Device::open_device_by_ip(ip).unwrap();

            println!(
                "model: {}, IP: {}, firmware: {}",
                CStr::from_ptr(uri)
                    .to_str()
                    .unwrap()
                    .split(":")
                    .collect::<Vec<&str>>()[0],
                CStr::from_ptr(ip).to_str().unwrap(),
                device.get_firmware_version().expect("Cannot get firmware version"),
            );

            let mut data_mode = sys::PsDataMode::default();
            status = sys::Ps2_GetDataMode(device.handle, SESSION_INDEX, &mut data_mode);
            if status != OK {
                return Err(format!("get data mode failed with status {}", status));
            }

            status = sys::Ps2_StartStream(device.handle, SESSION_INDEX);
            if status != OK {
                return Err(format!("start stream failed with status {}", status));
            } else {
                println!("stream started, data mode: {}", data_mode);
            }

            sys::Ps2_SetRGBResolution(
                device.handle,
                SESSION_INDEX,
                sys::PsResolution_PsRGB_Resolution_640_480,
            );

            Ok(device)
        }
    }

    fn get_firmware_version(&self) -> Result<String, String> {
        let mut buffer = [0; 64];
        match get_firmware_version(self.handle, &mut buffer) {
            OK => Ok(CStr::from_bytes_until_nul(&buffer).unwrap().to_string_lossy().into_owned()),
            error_code => Err(format!("{}", error_code)),
        }
    }

    fn open_device_by_ip(ip: *const c_char) -> Result<Self, String> {
        let mut handle = 0 as sys::PsDeviceHandle;
        let status = unsafe { sys::Ps2_OpenDeviceByIP(ip, &mut handle) };
        if status != OK {
            return Err(format!("open device failed with status {}", status));
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
            Err("device ptr is null".to_string())
        }
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
            println!("!!! pixel count is not equal to {} * {}", w, h)
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
                "setting of color resolution is ignored because color frame is mapped to depth"
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
            _ => panic!("unknown rgb resolution"),
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
        unsafe {
            let mut depth_range = sys::PsDepthRange::default();

            sys::Ps2_GetDepthRange(self.handle, SESSION_INDEX, &mut depth_range);

            let mut mr = sys::PsMeasuringRange::default();

            sys::Ps2_GetMeasuringRange(self.handle, SESSION_INDEX, depth_range, &mut mr);

            match depth_range {
                0 => (mr.effectDepthMinNear, mr.effectDepthMaxNear),
                1 => (mr.effectDepthMinMid, mr.effectDepthMaxMid),
                2 => (mr.effectDepthMinFar, mr.effectDepthMaxFar),
                _ => panic!("unknown measuring range"),
            }
        }
    }

    /// Stops the stream, closes the device, and clears all resources.
    pub fn shut_down(&mut self) {
        unsafe {
            sys::Ps2_StopStream(self.handle, SESSION_INDEX);
            sys::Ps2_CloseDevice(&mut self.handle);

            let status = sys::Ps2_Shutdown();
            if status != OK {
                println!("shut down failed with status: {}", status);
            } else {
                println!("shut down device successfully");
            }
        }
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