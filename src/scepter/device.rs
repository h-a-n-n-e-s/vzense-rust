//! Basic routines to initialize or shut down a device.

use std::ffi::CStr;
use sys::ScStatus_SC_OK as OK;

use vzense_sys::scepter as sys;

use crate::{ColorFormat, ColorResolution, FrameType, Resolution, DEFAULT_RESOLUTION};

/// Device is a wrapper for the raw pointer `handle` used in every `sys` function. It also contains `frame_ready` (containing frame availability data) and `frame` (containing frame data).
pub struct Device {
    pub(super) handle: sys::ScDeviceHandle,
    pub(super) frame_ready: sys::ScFrameReady,
    pub(super) frame: sys::ScFrame,
    pub(super) current_frame_type: Option<FrameType>,
    pub(super) min_depth_mm: u16,
    pub(super) max_depth_mm: u16,
}
impl Device {
    /// Initializes the sytem and returns a device if it finds one. Make sure a Vzense camera is connected. After 3 seconds the routine will time out if no device was found.
    pub fn init() -> Result<Self, String> {
        unsafe {
            println!("initializing...");

            let mut status = sys::scInitialize();
            if status != OK {
                return Err(format!("initialization failed with status {}", status));
            }

            let mut device_count = 0;
            println!("searching for device... ");
            status = sys::scGetDeviceCount(&mut device_count, 3000);
            if status != OK {
                return Err(format!("get device count failed with status {}", status));
            } else if device_count > 0 {
                println!("device found");
            } else {
                return Err("no device found".to_string());
            }

            let mut device_info = sys::ScDeviceInfo::default();

            sys::scGetDeviceInfoList(device_count, &mut device_info);
            let ip = device_info.ip.as_ptr();
            let model = device_info.productName.as_ptr();

            // Valgrind gets stuck between these two marks
            // println!("----------------------------------------------1");

            let device = Device::open_device_by_ip(ip).unwrap();

            // println!("----------------------------------------------2");

            let mut firmware = [0; 64];
            status = sys::scGetFirmwareVersion(device.handle, firmware.as_mut_ptr(), 64);
            if status != OK {
                return Err(format!(
                    "get firmware version failed with status {}",
                    status
                ));
            }

            println!(
                "model: {}, IP: {}, firmware: {}",
                CStr::from_ptr(model).to_str().unwrap(),
                CStr::from_ptr(ip).to_str().unwrap(),
                CStr::from_bytes_until_nul(std::mem::transmute::<&[i8], &[u8]>(&firmware))
                    .unwrap()
                    .to_str()
                    .unwrap()
            );

            let mut work_mode = sys::ScWorkMode::default();
            status = sys::scGetWorkMode(device.handle, &mut work_mode);
            if status != OK {
                return Err(format!("get work mode failed with status {}", status));
            }

            status = sys::scStartStream(device.handle);
            if status != OK {
                return Err(format!("start stream failed with status {}", status));
            } else {
                println!("stream started, work mode: {}", work_mode);
            }

            Ok(device)
        }
    }

    fn open_device_by_ip(ip: *const i8) -> Result<Self, String> {
        let mut handle = 0 as sys::ScDeviceHandle;
        let status = unsafe { sys::scOpenDeviceByIP(ip, &mut handle) };
        if status != OK {
            return Err(format!("open device failed with status {}", status));
        }
        if !handle.is_null() {
            Ok(Device {
                handle,
                frame_ready: sys::ScFrameReady::default(),
                frame: sys::ScFrame::default(),
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

    pub fn get_frame_info(&self) -> sys::ScFrame {
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
            sys::scSetTransformColorImgToDepthSensorEnabled(
                self.handle,
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

    /// Sets the resolution of the color frame. Three resolutions are currently available: 640x480, 800x600, and 1600x1200.
    pub fn set_color_resolution(&self, resolution: ColorResolution) {
        unsafe {
            // check if color is mapped to depth
            let mut is_mapped = 0;
            sys::scGetTransformColorImgToDepthSensorEnabled(self.handle, &mut is_mapped);
            if is_mapped == 1 && resolution != ColorResolution::Res640x480 {
                sys::scSetColorResolution(self.handle, 640, 480);
                println!(
                    "setting of color resolution is ignored because color frame is mapped to depth"
                )
            }

            match resolution {
                ColorResolution::Res640x480 => sys::scSetColorResolution(self.handle, 640, 480),
                ColorResolution::Res800x600 => sys::scSetColorResolution(self.handle, 800, 600),
                ColorResolution::Res1600x1200 => sys::scSetColorResolution(self.handle, 1600, 1200),
            };
        }
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

    /// [depricated] Returns the current depth range `(min, max)` of the camera in mm. Note: At least the min value seems to have no practical meaning. For the NYX650 the returned min value is 1 mm which makes no sense, while the max value is 4700 mm. In the specs the depth range for the NYX650 is given as min: 300 mm, max: 4500 mm.
    #[deprecated]
    pub fn get_depth_measuring_range(&self) -> (u16, u16) {
        unsafe {
            let mut min = 0;
            let mut max = 0;
            sys::scGetDepthRangeValue(self.handle, &mut min, &mut max);
            (min as u16, max as u16)
        }
    }

    /// Stops the stream, closes the device, and clears all resources.
    pub fn shut_down(&mut self) {
        unsafe {
            sys::scStopStream(self.handle);
            sys::scCloseDevice(&mut self.handle);

            let status = sys::scShutdown();
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
