use crate::SESSION_INDEX;

use std::{ffi::CStr, thread::sleep, time::Duration};
use sys::PsReturnStatus_PsRetOK as ok;
use vzense_sys as sys;

pub fn init() -> Result<sys::PsDeviceHandle, String> {
    unsafe {
        println!("initializing...");
        let mut status = sys::Ps2_Initialize();
        if status != ok {
            return Err(format!("initialization failed with status {}", status));
        }
        let device_count = &mut 0;
        let mut times_tried = 0;
        println!("searching for device...");
        loop {
            status = sys::Ps2_GetDeviceCount(device_count);
            if status != ok {
                return Err(format!("get device count failed with status {}", status));
            } else {
                if *device_count > 0 {
                    print!("device found, ");
                    break;
                }
                times_tried += 1;
                // give up after 10 seconds
                if times_tried == 50 {
                    return Err(format!("no device found"));
                }
                sleep(Duration::from_millis(200));
            }
        }

        let device_info = &mut sys::PsDeviceInfo::default();

        sys::Ps2_GetDeviceListInfo(device_info, *device_count);
        let uri = device_info.uri.as_ptr();
        println!("uri: {}", CStr::from_ptr(uri).to_str().unwrap());

        let device_handle = &mut (0 as sys::PsDeviceHandle);
        status = sys::Ps2_OpenDevice(uri, device_handle);
        if status != ok {
            return Err(format!("open device failed with status {}", status));
        }

        status = sys::Ps2_StartStream(*device_handle, SESSION_INDEX);
        if status != ok {
            return Err(format!("start stream failed with status {}", status));
        } else {
            println!("stream started");
        }

        let data_mode = &mut sys::PsDataMode::default();
        status = sys::Ps2_GetDataMode(*device_handle, SESSION_INDEX, data_mode);
        if status != ok {
            return Err(format!("get data mode failed with status {}", status));
        } else {
            println!("data mode: {}", *data_mode);
        }

        Ok(*device_handle)
    }
}

pub fn get_measuring_range(device_handle: sys::PsDeviceHandle) -> sys::PsMeasuringRange {
    unsafe {
        let depth_range = &mut sys::PsDepthRange::default();

        sys::Ps2_GetDepthRange(device_handle, SESSION_INDEX, depth_range);

        let measuring_range = &mut sys::PsMeasuringRange::default();

        sys::Ps2_GetMeasuringRange(device_handle, SESSION_INDEX, *depth_range, measuring_range);

        *measuring_range
    }
}
