//! Reading frames, creating data arrays.

use crate::FrameType;

use super::SESSION_INDEX;

use super::device::Device;
use std::iter::zip;
use sys::PsReturnStatus_PsRetOK as OK;
use vzense_sys::dcam560 as sys;

/// Flag signaling if a frame is available
pub type FrameReady = sys::PsFrameReady;

/// Depth/IR/RGB image frame data.
pub type Frame = sys::PsFrame;

/// Captures the next image frame from `device`. This API must be invoked before capturing frame data using `get_frame()`. `frame_ready` is a pointer to a buffer storing the signal for the frame availability.
pub fn read_next_frame(device: &mut Device) -> i32 {
    unsafe {
        let status = sys::Ps2_ReadNextFrame(device.handle, SESSION_INDEX, &mut device.frame_ready);
        if status != OK {
            println!("vzense_rust: read_next_frame failed with status {}", status);
            return status;
        }
    }
    0
}

/// Returns the image data in `frame` for the current frame from `device`. Before invoking this API, invoke `read_next_frame()` to capture one image frame from the device. `frame_ready` is a pointer to a buffer storing the signal for the frame availability set in `read_next_frame()`.
pub fn get_frame(device: &mut Device, frame_type: &FrameType, out: &mut [u8]) {
    let mut ft: Option<sys::PsFrameType> = None;
    match frame_type {
        FrameType::Depth => {
            if device.frame_ready.depth() == 1 {
                ft = Some(sys::PsFrameType_PsDepthFrame);
            }
        }
        FrameType::IR => {
            if device.frame_ready.ir() == 1 {
                ft = Some(sys::PsFrameType_PsIRFrame);
            }
        }
        FrameType::Color => {
            if device.frame_ready.rgb() == 1 {
                ft = Some(sys::PsFrameType_PsRGBFrame);
            }
        }
        FrameType::ColorMapped => {
            if device.frame_ready.mappedRGB() == 1 {
                ft = Some(sys::PsFrameType_PsMappedRGBFrame);
            }
        }
    }
    if let Some(ft) = ft {
        unsafe {
            let status = sys::Ps2_GetFrame(device.handle, SESSION_INDEX, ft, &mut device.frame);
            if status != OK {
                panic!("get_frame failed with status {}", status);
            }
        }
        if device.frame.pFrameData.is_null() {
            panic!("frame pointer is NULL!");
        }
        match frame_type {
            FrameType::Depth => {
                get_normalized_depth(device, out);
                device.current_frame_type = Some(FrameType::Depth);
            }
            FrameType::IR => {
                get_normalized_ir(device, 0, 255, out);
                device.current_frame_type = Some(FrameType::IR);
            }
            FrameType::Color | FrameType::ColorMapped => {
                get_bgr(device, out);
                if *frame_type == FrameType::Color {
                    device.current_frame_type = Some(FrameType::Color);
                } else {
                    device.current_frame_type = Some(FrameType::ColorMapped);
                }
            }
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
