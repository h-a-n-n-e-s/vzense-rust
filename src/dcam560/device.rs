//! Basic routines to initialize or shut down a device.

use std::{ffi::CStr, thread::sleep, time::Duration};

use sys::PsReturnStatus_PsRetOK as OK;
use vzense_sys::dcam560 as sys;

use crate::util::{RGBResolution, Resolution, DEFAULT_RESOLUTION};

use super::SESSION_INDEX;

/// Device to connect to.
pub type Device = sys::PsDeviceHandle;

/// Possible depth ranges.
pub enum DepthRange {
    Near,
    Mid,
    Far,
}

/// Initializes the sytem and returns a device if it finds one. Make sure a Vzense camera is connected. After 3 seconds the routine will time out if no device was found.
pub fn init() -> Result<Device, String> {
    unsafe {
        println!("initializing...");
        let mut status = sys::Ps2_Initialize();
        if status != OK {
            return Err(format!("initialization failed with status {}", status));
        }
        let device_count = &mut 0;
        let mut times_tried = 0;
        println!("searching for device...");
        loop {
            status = sys::Ps2_GetDeviceCount(device_count);
            if status != OK {
                return Err(format!("get device count failed with status {}", status));
            } else {
                if *device_count > 0 {
                    print!("device found, ");
                    break;
                }
                times_tried += 1;
                // give up after 3 seconds
                if times_tried == 15 {
                    return Err(format!("no device found"));
                }
                sleep(Duration::from_millis(200));
            }
        }

        let device_info = &mut sys::PsDeviceInfo::default();

        sys::Ps2_GetDeviceListInfo(device_info, *device_count);
        let uri = device_info.uri.as_ptr();
        let ip = device_info.ip.as_ptr();
        println!(
            "uri: {}, IP: {}",
            CStr::from_ptr(uri).to_str().unwrap(),
            CStr::from_ptr(ip).to_str().unwrap()
        );

        let device = &mut (0 as sys::PsDeviceHandle);
        status = sys::Ps2_OpenDevice(uri, device);
        if status != OK {
            return Err(format!("open device failed with status {}", status));
        }

        status = sys::Ps2_StartStream(*device, SESSION_INDEX);
        if status != OK {
            return Err(format!("start stream failed with status {}", status));
        } else {
            println!("stream started");
        }

        let data_mode = &mut sys::PsDataMode::default();
        status = sys::Ps2_GetDataMode(*device, SESSION_INDEX, data_mode);
        if status != OK {
            return Err(format!("get data mode failed with status {}", status));
        } else {
            println!("data mode: {}", *data_mode);
        }

        Ok(*device)
    }
}

/// Enable or disable the mapping of RGB image to depth camera space.
pub fn map_rgb_to_depth(device: Device, is_enabled: bool) {
    let rgb_resolution = get_rgb_resolution(device);
    if rgb_resolution != DEFAULT_RESOLUTION {
        set_rgb_resolution(device, RGBResolution::RGBRes640x480);
    }
    unsafe {
        // should actually be `Ps2_SetMapperEnabledRGBToDepth` but the names seem to be mixed up
        sys::Ps2_SetMapperEnabledDepthToRGB(device, SESSION_INDEX, if is_enabled { 1 } else { 0 });
    }
}

/// Sets the resolution of the rgb frame. Three resolutions are currently available: 640x480, 800x600, and 1600x1200.
pub fn set_rgb_resolution(device: Device, resolution: RGBResolution) {
    unsafe {
        let mut resolution = match resolution {
            RGBResolution::RGBRes640x480 => sys::PsResolution_PsRGB_Resolution_640_480,
            RGBResolution::RGBRes800x600 => sys::PsResolution_PsRGB_Resolution_800_600,
            RGBResolution::RGBRes1600x1200 => sys::PsResolution_PsRGB_Resolution_1600_1200,
        };

        // check if rgb is mapped to depth
        let is_mapped = &mut 0;
        // should actually be `Ps2_GetMapperEnabledRGBToDepth` but the names seem to be mixed up
        sys::Ps2_GetMapperEnabledDepthToRGB(device, SESSION_INDEX, is_mapped);

        if *is_mapped == 1 {
            resolution = sys::PsResolution_PsRGB_Resolution_640_480;
            println!("setting of rgb resolution is ignored because rgb frame is mapped to depth")
        }

        sys::Ps2_SetRGBResolution(device, SESSION_INDEX, resolution);
    }
}

/// Returns the resolution of the rgb frame.
pub fn get_rgb_resolution(device: Device) -> Resolution {
    let resolution_type = &mut 0;
    unsafe {
        sys::Ps2_GetRGBResolution(device, SESSION_INDEX, resolution_type);
    }
    match *resolution_type {
        2 => Resolution::new(640, 480),
        5 => Resolution::new(800, 600),
        4 => Resolution::new(1600, 1200),
        _ => panic!("unknown rgb resolution"),
    }
}

/// Sets the depth range mode.
pub fn set_depth_measuring_range_dcam560(device: Device, depth_range: DepthRange) {
    let depth_range = match depth_range {
        DepthRange::Near => 0,
        DepthRange::Mid => 1,
        DepthRange::Far => 2,
    };
    unsafe {
        sys::Ps2_SetDepthRange(device, SESSION_INDEX, depth_range);
    }
}

/// Returns the current measuring range `(min, max)` of the camera in mm
pub fn get_depth_measuring_range(device: Device) -> (u16, u16) {
    unsafe {
        let depth_range = &mut sys::PsDepthRange::default();

        sys::Ps2_GetDepthRange(device, SESSION_INDEX, depth_range);

        let mr = &mut sys::PsMeasuringRange::default();

        sys::Ps2_GetMeasuringRange(device, SESSION_INDEX, *depth_range, mr);

        match *depth_range {
            0 => (mr.effectDepthMinNear, mr.effectDepthMaxNear),
            1 => (mr.effectDepthMinMid, mr.effectDepthMaxMid),
            2 => (mr.effectDepthMinFar, mr.effectDepthMaxFar),
            _ => panic!("unknown measuring range"),
        }
    }
}

/// Stops the stream, closes the device, and clears all resources.
pub fn shut_down(device: &mut Device) {
    unsafe {
        sys::Ps2_StopStream(*device, SESSION_INDEX);
        sys::Ps2_CloseDevice(device);

        let status = sys::Ps2_Shutdown();
        if status != OK {
            println!("shut down failed with status: {}", status);
        } else {
            println!("shut down device successfully");
        }
    }
}
