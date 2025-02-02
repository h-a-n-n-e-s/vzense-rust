//! Reading frames, retrieving data.

use crate::util::{new_fixed_vec, normalize_u16_to_u8};

use super::SESSION_INDEX;

use super::device::Device;
use std::iter::zip;
use sys::PsReturnStatus_PsRetOK as OK;
use vzense_sys::dcam560 as sys;

/// Captures the next image frame from `device`. This function must be called before getting frame data using `get_..._frame()`.
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

/// Raw depth data in mm as `u16`.
pub fn get_depth_mm_u16_frame(device: &mut Device, depth_mm: &mut [u16]) {
    if device.frame_ready.depth() == 1 {
        let status = unsafe {
            sys::Ps2_GetFrame(
                device.handle,
                SESSION_INDEX,
                sys::PsFrameType_PsDepthFrame,
                &mut device.frame,
            )
        };
        check_frame(device, status);
        get_depth_mm(device, depth_mm);

        device.current_frame_is_depth = true;
    }
}

/// Depth data scaled according to `device.min_depth_mm` = 0 and `device.max_depth_mm` = 255 stored in a `u8` array.
pub fn get_depth_scaled_u8_frame(device: &mut Device, depth_scaled: &mut [u8]) {
    if device.frame_ready.depth() == 1 {
        let status = unsafe {
            sys::Ps2_GetFrame(
                device.handle,
                SESSION_INDEX,
                sys::PsFrameType_PsDepthFrame,
                &mut device.frame,
            )
        };
        check_frame(device, status);
        let mut depth_mm = new_fixed_vec(depth_scaled.len(), 0);

        get_depth_mm(device, &mut depth_mm);

        normalize_u16_to_u8(
            &depth_mm,
            device.min_depth_mm,
            device.max_depth_mm,
            depth_scaled,
        );
        device.current_frame_is_depth = true;
    }
}

/// Frame contains no IR data, even if
/// sys::Ps2_SetDataMode(.., .., sys::PsDataMode_PsIRAndRGB_30)
/// is set.
pub fn get_ir_scaled_u8_frame(device: &mut Device, ir_scaled: &mut [u8]) {
    if device.frame_ready.ir() == 1 {
        let status = unsafe {
            sys::Ps2_GetFrame(
                device.handle,
                SESSION_INDEX,
                sys::PsFrameType_PsIRFrame,
                &mut device.frame,
            )
        };
        check_frame(device, status);
        get_normalized_ir(device, 0, 255, ir_scaled);
        device.current_frame_is_depth = false;
    }
}

/// Color data as 24 bit stored in consecutive `u8`.
pub fn get_color_frame(device: &mut Device, color: &mut [u8]) {
    let frame_type = if device.color_is_mapped && device.frame_ready.mappedRGB() == 1 {
        sys::PsFrameType_PsMappedRGBFrame
    } else if !device.color_is_mapped && device.frame_ready.rgb() == 1 {
        sys::PsFrameType_PsRGBFrame
    } else {
        return;
    };
    let status =
        unsafe { sys::Ps2_GetFrame(device.handle, SESSION_INDEX, frame_type, &mut device.frame) };
    check_frame(device, status);
    get_color(device, color);
    device.current_frame_is_depth = false;
}

fn get_depth_mm(device: &Device, depth_mm: &mut [u16]) {
    let p = unsafe {
        std::ptr::slice_from_raw_parts(device.frame.pFrameData, device.frame.dataLen as usize)
            .as_ref()
            .unwrap()
    };
    for (dmmi, pi) in zip(depth_mm, p.chunks_exact(2)) {
        // create one u16 from two consecutive u8 values
        *dmmi = u16::from_le_bytes([pi[0], pi[1]]);
    }
}

fn get_normalized_ir(device: &Device, min_ir: u16, max_ir: u16, normalized_ir: &mut [u8]) {
    unsafe {
        let p =
            std::ptr::slice_from_raw_parts(device.frame.pFrameData, device.frame.dataLen as usize)
                .as_ref()
                .unwrap();

        for (nii, pi) in zip(normalized_ir, p.chunks_exact(2)) {
            // create one u16 from two consecutive u8 values
            let tmp = u16::from_le_bytes([pi[0], pi[1]]);
            // scale to u8
            *nii = ((tmp - min_ir) as f32 * 255.0 / (max_ir - min_ir) as f32).floor() as u8;
        }
    }
}

fn get_color(device: &Device, color: &mut [u8]) {
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

fn check_frame(device: &Device, status: sys::PsReturnStatus) {
    if status != OK {
        panic!("get_frame failed with status {}", status);
    }
    if device.frame.pFrameData.is_null() {
        panic!("frame pointer is NULL!");
    }
}
