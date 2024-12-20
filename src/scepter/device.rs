//! Basic routines to initialize or shut down a device.

use std::ffi::CStr;

use sys::ScStatus_SC_OK as OK;
use vzense_sys::scepter as sys;

use crate::util::{RGBResolution, Resolution, DEFAULT_RESOLUTION};

/// Device is a wrapper for the raw pointer `handle`
pub struct Device {
    pub handle: sys::ScDeviceHandle,
}
impl Device {
    fn open_device_by_ip(ip: *const i8) -> Result<Self, String> {
        let handle = &mut (0 as sys::ScDeviceHandle);
        let status = unsafe { sys::scOpenDeviceByIP(ip, handle) };
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

/// Initializes the sytem and returns a device if it finds one. Make sure a Vzense camera is connected. After 3 seconds the routine will time out if no device was found.
pub fn init() -> Result<Device, String> {
    unsafe {
        println!("initializing...");
        let mut status = sys::scInitialize();
        if status != OK {
            return Err(format!("initialization failed with status {}", status));
        }
        let device_count = &mut 0;
        println!("searching for device...");
        status = sys::scGetDeviceCount(device_count, 3000);
        if status != OK {
            return Err(format!("get device count failed with status {}", status));
        } else if *device_count > 0 {
            print!("device found, ");
        } else {
            return Err("no device found".to_string());
        }

        let device_info = &mut sys::ScDeviceInfo::default();

        sys::scGetDeviceInfoList(*device_count, device_info);
        let ip = device_info.ip.as_ptr();
        let model = device_info.productName.as_ptr();
        println!(
            "model: {}, IP: {}",
            CStr::from_ptr(model).to_str().unwrap(),
            CStr::from_ptr(ip).to_str().unwrap()
        );

        let device = Device::open_device_by_ip(ip).unwrap();

        status = sys::scStartStream(device.handle);
        if status != OK {
            return Err(format!("start stream failed with status {}", status));
        } else {
            println!("stream started");
        }

        let work_mode = &mut sys::ScWorkMode::default();
        status = sys::scGetWorkMode(device.handle, work_mode);
        if status != OK {
            return Err(format!("get work mode failed with status {}", status));
        } else {
            println!("work mode: {}", *work_mode);
        }

        Ok(device)
    }
}

/// Enable or disable the mapping of RGB image to depth camera space.
pub fn map_rgb_to_depth(device: &Device, is_enabled: bool) {
    let rgb_resolution = get_rgb_resolution(device);
    if rgb_resolution != DEFAULT_RESOLUTION {
        set_rgb_resolution(device, RGBResolution::RGBRes640x480);
    }
    unsafe {
        // sys::scSetTransformDepthImgToColorSensorEnabled(device, if is_enabled { 1 } else { 0 });
        sys::scSetTransformColorImgToDepthSensorEnabled(
            device.handle,
            if is_enabled { 1 } else { 0 },
        );
    }
}

/// Sets the resolution of the rgb frame. Three resolutions are currently available: 640x480, 800x600, and 1600x1200.
pub fn set_rgb_resolution(device: &Device, resolution: RGBResolution) {
    unsafe {
        // check if rgb is mapped to depth
        let is_mapped = &mut 0;
        // sys::scGetTransformDepthImgToColorSensorEnabled(device, is_mapped);
        sys::scGetTransformColorImgToDepthSensorEnabled(device.handle, is_mapped);
        if *is_mapped == 1 && resolution != RGBResolution::RGBRes640x480 {
            sys::scSetColorResolution(device.handle, 640, 480);
            println!("setting of rgb resolution is ignored because rgb frame is mapped to depth")
        }

        match resolution {
            RGBResolution::RGBRes640x480 => sys::scSetColorResolution(device.handle, 640, 480),
            RGBResolution::RGBRes800x600 => sys::scSetColorResolution(device.handle, 800, 600),
            RGBResolution::RGBRes1600x1200 => sys::scSetColorResolution(device.handle, 1600, 1200),
        };
    }
}

/// Returns the resolution of the rgb frame.
pub fn get_rgb_resolution(device: &Device) -> Resolution {
    let w = &mut 0;
    let h = &mut 0;
    unsafe {
        sys::scGetColorResolution(device.handle, w, h);
    }
    Resolution::new(*w as u32, *h as u32)
}

/// Returns the current depth range `(min, max)` of the camera in mm. Note: At least the min value seems to have no practical meaning. For the NYX650 the returned min value is 1 mm which makes no sense, while the max value is 4700 mm. In the specs the depth range for the NYX650 is given as min: 300 mm, max: 4500 mm.
pub fn get_depth_measuring_range(device: &Device) -> (u16, u16) {
    unsafe {
        let min = &mut 0;
        let max = &mut 0;
        sys::scGetDepthRangeValue(device.handle, min, max);
        (*min as u16, *max as u16)
    }
}

/// Stops the stream, closes the device, and clears all resources.
pub fn shut_down(device: &mut Device) {
    unsafe {
        sys::scStopStream(device.handle);
        sys::scCloseDevice(&mut device.handle);

        let status = sys::scShutdown();
        if status != OK {
            println!("shut down failed with status: {}", status);
        } else {
            println!("shut down device successfully");
        }
    }
}
