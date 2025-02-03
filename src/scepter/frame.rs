//! Reading frames, retrieving data.

use crate::util::{new_fixed_vec, normalize_u16_to_u8};

use super::{device::Device, get_message};
use std::iter::zip;
use sys::ScStatus_SC_OK as OK;
use vzense_sys::scepter as sys;

/// Captures the next image frame from `device`. This function must be called before getting frame data using `get_..._frame()`. `max_wait_time_ms` is the maximum waiting time for the next frame in milliseconds. The recommended value is 2000 / fps.
pub fn read_next_frame(device: &mut Device, max_wait_time_ms: u16) -> i32 {
    unsafe {
        let status = sys::scGetFrameReady(device.handle, max_wait_time_ms, &mut device.frame_ready);
        if status != OK {
            println!(
                "vzense_rust: read_next_frame failed with status {}",
                get_message(status)
            );
            return status;
        }
    }
    0
}

/// Raw depth data in mm as `u16`.
pub fn get_depth_mm_u16_frame(device: &mut Device, depth_mm: &mut [u16]) {
    if device.frame_ready.depth() == 1 {
        let status = unsafe {
            sys::scGetFrame(
                device.handle,
                sys::ScFrameType_SC_DEPTH_FRAME,
                &mut device.frame,
            )
        };
        check_frame(device, status);
        get_u16_data(device, depth_mm);

        device.current_frame_is_depth = true;
    }
}

/// Depth data scaled according to `device.min_depth_mm` = 0 and `device.max_depth_mm` = 255 stored in a `u8` array.
pub fn get_depth_scaled_u8_frame(device: &mut Device, depth_scaled: &mut [u8]) {
    if device.frame_ready.depth() == 1 {
        let status = unsafe {
            sys::scGetFrame(
                device.handle,
                sys::ScFrameType_SC_DEPTH_FRAME,
                &mut device.frame,
            )
        };
        check_frame(device, status);
        let mut depth_mm = new_fixed_vec(depth_scaled.len(), 0);

        get_u16_data(device, &mut depth_mm);

        normalize_u16_to_u8(
            &depth_mm,
            device.min_depth_mm,
            device.max_depth_mm,
            depth_scaled,
        );
        device.current_frame_is_depth = true;
    }
}

/// Raw IR data as `u8`.
pub fn get_ir_frame(device: &mut Device, ir: &mut [u8]) {
    if device.frame_ready.ir() == 1 {
        let status = unsafe {
            sys::scGetFrame(
                device.handle,
                sys::ScFrameType_SC_IR_FRAME,
                &mut device.frame,
            )
        };
        check_frame(device, status);
        get_u8_data(device, ir);
        device.current_frame_is_depth = false;
    }
}

/// Color data as 24 bit stored in consecutive `u8`.
pub fn get_color_frame(device: &mut Device, color: &mut [u8]) {
    let frame_type = if device.color_is_mapped && device.frame_ready.transformedColor() == 1 {
        sys::ScFrameType_SC_TRANSFORM_COLOR_IMG_TO_DEPTH_SENSOR_FRAME
    } else if !device.color_is_mapped && device.frame_ready.color() == 1 {
        sys::ScFrameType_SC_COLOR_FRAME
    } else {
        return;
    };
    let status = unsafe { sys::scGetFrame(device.handle, frame_type, &mut device.frame) };
    check_frame(device, status);
    get_u8_data(device, color);
    device.current_frame_is_depth = false;
}

fn get_u16_data(device: &Device, data: &mut [u16]) {
    let p = unsafe {
        std::ptr::slice_from_raw_parts(device.frame.pFrameData, device.frame.dataLen as usize)
            .as_ref()
            .unwrap()
    };
    for (di, pi) in zip(data, p.chunks_exact(2)) {
        // create one u16 from two consecutive u8 values
        *di = u16::from_le_bytes([pi[0], pi[1]]);
    }
}

fn get_u8_data(device: &Device, data: &mut [u8]) {
    unsafe {
        let p =
            std::ptr::slice_from_raw_parts(device.frame.pFrameData, device.frame.dataLen as usize)
                .as_ref()
                .unwrap();

        if data.len() == (*p).len() {
            data.copy_from_slice(p);
        }
    }
}

/// Check status of `scGetFrame()` and if data pointer is null.
fn check_frame(device: &Device, status: sys::ScStatus) {
    if status != OK {
        panic!("get_frame failed with status {}", get_message(status));
    }
    if device.frame.pFrameData.is_null() {
        panic!("frame pointer is NULL!");
    }
}
