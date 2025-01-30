//! Basic routines to initialize or shut down a device.

use std::{ffi::CStr, thread::sleep, time::Duration};

use sys::PsReturnStatus_PsRetOK as OK;
use vzense_sys::dcam560 as sys;

use crate::{ColorFormat, ColorResolution, DepthRange, FrameType, Resolution, DEFAULT_RESOLUTION};

use super::SESSION_INDEX;

/// Device is a wrapper for the raw pointer `handle` used in every `sys` function. It also contains `frame_ready` (containing frame availability data) and `frame` (containing frame data).
pub struct Device {
    pub(super) handle: sys::PsDeviceHandle,
    pub(super) frame_ready: sys::PsFrameReady,
    pub(super) frame: sys::PsFrame,
    pub(super) current_frame_type: Option<FrameType>,
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
            let ip = device_info.ip.as_ptr();
            let uri = device_info.uri; // model_name:serial_number
                                       // let alias = device_info.alias; // serial number

            let device = Device::open_device_by_ip(ip).unwrap();

            let mut firmware = [0; 64];
            status = sys::Ps2_GetFirmwareVersionNumber(
                device.handle,
                SESSION_INDEX,
                firmware.as_mut_ptr(),
                64,
            );
            if status != OK {
                return Err(format!(
                    "get firmware version failed with status {}",
                    status
                ));
            }

            println!(
                "model: {}, IP: {}, firmware: {}",
                CStr::from_bytes_until_nul(std::mem::transmute::<&[i8], &[u8]>(&uri))
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split(":")
                    .collect::<Vec<&str>>()[0],
                CStr::from_ptr(ip).to_str().unwrap(),
                CStr::from_bytes_until_nul(std::mem::transmute::<&[i8], &[u8]>(&firmware))
                    .unwrap()
                    .to_str()
                    .unwrap()
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

            Ok(device)
        }
    }

    fn open_device_by_ip(ip: *const i8) -> Result<Self, String> {
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
                current_frame_type: None,
                min_depth_mm: 500,  // default value
                max_depth_mm: 1000, // default value
            })
        } else {
            Err("device ptr is null".to_string())
        }
    }

    pub fn set_depth_range(&mut self, min_depth_mm: u16, max_depth_mm: u16) {
        self.min_depth_mm = min_depth_mm;
        self.max_depth_mm = max_depth_mm;
    }

    pub fn get_frame_info(&self) -> sys::PsFrame {
        self.frame
    }

    /// Checks if the number of pixels in `frame` equals `pixel_count`.
    pub fn check_pixel_count(&self, pixel_count: usize) {
        let w = self.frame.width as usize;
        let h = self.frame.height as usize;
        if w * h != pixel_count {
            println!("!!! pixel count is not equal to {} * {}", w, h)
        }
    }

    /// Enable or disable the mapping of the color image to depth camera space.
    pub fn map_color_to_depth(&self, is_enabled: bool) {
        let color_resolution = self.get_color_resolution();
        if color_resolution != DEFAULT_RESOLUTION {
            self.set_color_resolution(ColorResolution::Res640x480);
        }
        unsafe {
            // should actually be `Ps2_SetMapperEnabledRGBToDepth` but the names seem to be mixed up
            sys::Ps2_SetMapperEnabledDepthToRGB(
                self.handle,
                SESSION_INDEX,
                if is_enabled { 1 } else { 0 },
            );
            // let mut is_mapped = 0;
            // sys::scGetTransformColorImgToDepthSensorEnabled(self.handle, &mut is_mapped);
            // println!("is_mapped: {}", is_mapped);
        }
    }

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

    /// Sets the resolution of the color frame. Three resolutions are currently available: 640x480, 800x600, and 1600x1200.
    pub fn set_color_resolution(&self, resolution: ColorResolution) {
        unsafe {
            let mut resolution = match resolution {
                ColorResolution::Res640x480 => sys::PsResolution_PsRGB_Resolution_640_480,
                ColorResolution::Res800x600 => sys::PsResolution_PsRGB_Resolution_800_600,
                ColorResolution::Res1600x1200 => sys::PsResolution_PsRGB_Resolution_1600_1200,
            };

            // check if rgb is mapped to depth
            let mut is_mapped = 0;
            // should actually be `Ps2_GetMapperEnabledRGBToDepth` but the names seem to be mixed up
            sys::Ps2_GetMapperEnabledDepthToRGB(self.handle, SESSION_INDEX, &mut is_mapped);

            if is_mapped == 1 {
                resolution = sys::PsResolution_PsRGB_Resolution_640_480;
                println!(
                    "setting of rgb resolution is ignored because color frame is mapped to depth"
                )
            }

            sys::Ps2_SetRGBResolution(self.handle, SESSION_INDEX, resolution);
        }
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
    pub fn set_depth_measuring_range_dcam560(&self, depth_range: DepthRange) {
        let depth_range = match depth_range {
            DepthRange::Near => 0,
            DepthRange::Mid => 1,
            DepthRange::Far => 2,
        };
        unsafe {
            sys::Ps2_SetDepthRange(self.handle, SESSION_INDEX, depth_range);
        }
    }

    /// Returns the current measuring range `(min, max)` of the camera in mm
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

/// implement trait Data to allow use of Device in touch_detector
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
    fn get_current_frame_type(&self) -> &Option<FrameType> {
        &self.current_frame_type
    }
}

/*
/// Device is a wrapper for the raw pointer `handle`
pub struct Device {
    pub handle: sys::PsDeviceHandle,
}
impl Device {
    fn open_device_by_uri(uri: *const i8) -> Result<Self, String> {
        let handle = &mut (0 as sys::PsDeviceHandle);
        let status = unsafe { sys::Ps2_OpenDevice(uri, handle) };
        if status != OK {
            return Err(format!("open device failed with status {}", status));
        }
        if !handle.is_null() {
            Ok(Device { handle: *handle })
        } else {
            Err("device ptr is null".to_string())
        }
    }
}













*/
