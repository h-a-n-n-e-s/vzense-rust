//! Reading frames, creating data arrays.

use super::device::Device;
use std::iter::zip;
use vzense_sys::scepter as sys;

/// Flag signaling if a frame is available
pub type FrameReady = sys::ScFrameReady;

/// Depth/IR/Color image frame data.
pub type Frame = sys::ScFrame;

/// The available frame types, `Depth`, `IR` (infrared), `Color`, and `ColorMapped`.
pub enum FrameType {
    Depth,
    IR,
    Color,
    ColorMapped,
}

/// Captures the next image frame from `device`. This API must be invoked before capturing frame data using `get_frame()`. `max_wait_time_ms` is the maximum waiting time for the next frame in milliseconds. The recommended value is 2 * 1000 / fps. `frame_ready` is a pointer to a buffer storing the signal for the frame availability.
pub fn read_next_frame(device: &mut Device, max_wait_time_ms: u16) {
    unsafe {
        sys::scGetFrameReady(device.handle, max_wait_time_ms, &mut device.frame_ready);
    }
}

/// Returns the image data in `frame` for the current frame from `device`. Before invoking this API, invoke `read_next_frame()` to capture one image frame from the device. `frame_ready` is a pointer to a buffer storing the signal for the frame availability set in `read_next_frame()`.
pub fn get_frame(device: &mut Device, frame_type: &FrameType, out: &mut [u8]) {
    let mut ft: Option<sys::ScFrameType> = None;
    match frame_type {
        FrameType::Depth => {
            if device.frame_ready.depth() == 1 {
                ft = Some(sys::ScFrameType_SC_DEPTH_FRAME);
            }
        }
        FrameType::IR => {
            if device.frame_ready.ir() == 1 {
                ft = Some(sys::ScFrameType_SC_IR_FRAME);
            }
        }
        FrameType::Color => {
            if device.frame_ready.color() == 1 {
                ft = Some(sys::ScFrameType_SC_COLOR_FRAME);
            }
        }
        FrameType::ColorMapped => {
            if device.frame_ready.transformedColor() == 1 {
                ft = Some(sys::ScFrameType_SC_TRANSFORM_COLOR_IMG_TO_DEPTH_SENSOR_FRAME);
            }
        }
    }
    if let Some(ft) = ft {
        unsafe {
            let status = sys::scGetFrame(device.handle, ft, &mut device.frame);
            if status != sys::ScStatus_SC_OK {
                panic!("get_frame failed with status {}", status);
            }
        }
        if device.frame.pFrameData.is_null() {
            panic!("frame pointer is NULL!");
        }
        match frame_type {
            FrameType::Depth => get_normalized_depth(device, out),
            FrameType::IR => get_normalized_ir(device, 0, 255, out),
            FrameType::Color | FrameType::ColorMapped => get_bgr(device, out),
        }
    }
}

/// Creates `normalized_depth` data array from `frame`.
fn get_normalized_depth(device: &Device, normalized_depth: &mut [u8]) {
    unsafe {
        let p =
            std::ptr::slice_from_raw_parts(device.frame.pFrameData, device.frame.dataLen as usize)
                .as_ref()
                .unwrap();

        for (ndi, pi) in zip(normalized_depth, p.chunks_exact(2)) {
            // create one u16 from two consecutive u8 and clamp to measuring range
            let depth_mm =
                u16::from_le_bytes([pi[0], pi[1]]).clamp(device.min_depth_mm, device.max_depth_mm);

            // scale to u8
            *ndi = ((depth_mm - device.min_depth_mm) as f32 * 255.0
                / (device.max_depth_mm - device.min_depth_mm) as f32)
                .floor() as u8;
        }
    }
}

/// Creates `normalized_ir` data array from `frame`.
fn get_normalized_ir(device: &Device, min_ir: u8, max_ir: u8, normalized_ir: &mut [u8]) {
    unsafe {
        let p =
            std::ptr::slice_from_raw_parts(device.frame.pFrameData, device.frame.dataLen as usize)
                .as_ref()
                .unwrap();

        for (nii, pi) in zip(normalized_ir, p.iter()) {
            // scale to u8
            *nii = ((pi - min_ir) as f32 * 255.0 / (max_ir - min_ir) as f32).floor() as u8;
        }
    }
}

/// Creates `color` data array from `frame`.
fn get_bgr(device: &Device, color: &mut [u8]) {
    unsafe {
        let p =
            std::ptr::slice_from_raw_parts(device.frame.pFrameData, device.frame.dataLen as usize)
                .as_ref()
                .unwrap();

        if color.len() == (*p).len() {
            color.copy_from_slice(p);
        }
    }
}
