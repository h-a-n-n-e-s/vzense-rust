//! Reading frames, creating data arrays.

use std::iter::zip;

use super::SESSION_INDEX;

use super::device::Device;

use vzense_sys::dcam560 as sys;

/// Flag signaling if a frame is available
pub type FrameReady = sys::PsFrameReady;

/// Depth/IR/RGB image frame data.
pub type Frame = sys::PsFrame;

/// implement trait Data to allow use of type Frame in touch_detector
impl crate::util::touch_detector::Data for Frame {
    fn get_p_frame_data(&self) -> *mut u8 {
        self.pFrameData
    }
    fn get_data_len(&self) -> usize {
        self.dataLen as usize
    }
}

/// The available frame types, `Depth` and `RGB` (optical). IR frame is not implemented yet.
pub enum FrameType {
    /// depth
    Depth,
    /// optical
    RGB,
}

/// Captures the next image frame from `device`. This API must be invoked before capturing frame data using `get_frame()`. `frame_ready` is a pointer to a buffer storing the signal for the frame availability.
pub fn read_next_frame(device: Device, frame_ready: &mut FrameReady) {
    unsafe {
        sys::Ps2_ReadNextFrame(device, SESSION_INDEX, frame_ready);
    }
}

/// Returns the image data in `frame` for the current frame from `device`. Before invoking this API, invoke `read_next_frame()` to capture one image frame from the device. `frame_ready` is a pointer to a buffer storing the signal for the frame availability set in `read_next_frame()`. The image `frame_type` is either `FrameType::Depth` or `FrameType::RGB`.
pub fn get_frame(
    device: Device,
    frame_ready: &FrameReady,
    frame_type: FrameType,
    frame: &mut Frame,
) {
    unsafe {
        match frame_type {
            FrameType::Depth => {
                if frame_ready.depth() == 1 {
                    sys::Ps2_GetFrame(device, SESSION_INDEX, sys::PsFrameType_PsDepthFrame, frame);
                }
            }
            FrameType::RGB => {
                // check if rgb is mapped to depth
                let is_mapped = &mut 0;
                sys::Ps2_GetMapperEnabledDepthToRGB(device, SESSION_INDEX, is_mapped);

                let rgb_frame_type = match *is_mapped {
                    0 => sys::PsFrameType_PsRGBFrame,
                    _ => sys::PsFrameType_PsMappedRGBFrame,
                };

                if frame_ready.rgb() == 1 {
                    sys::Ps2_GetFrame(device, SESSION_INDEX, rgb_frame_type, frame);
                }
            }
        }
    }
}

/// Creates `normalized_depth` data array from `frame`.
pub fn get_normalized_depth(
    frame: &Frame,
    min_depth: u16,
    max_depth: u16,
    normalized_depth: &mut [u8],
) {
    unsafe {
        let p = std::ptr::slice_from_raw_parts(frame.pFrameData, frame.dataLen as usize)
            .as_ref()
            .unwrap();

        for (ndi, pi) in zip(normalized_depth, p.chunks_exact(2)) {
            // create one u16 from two consecutive u8 and clamp to measuring range
            let depth_mm = u16::from_le_bytes([pi[0], pi[1]]).clamp(min_depth, max_depth);

            // scale to u8
            *ndi = ((depth_mm - min_depth) as f32 * 255.0 / (max_depth - min_depth) as f32).floor()
                as u8;
        }
    }
}

/// Creates `bgr` data array from `frame`.
pub fn get_bgr(frame: &Frame, bgr: &mut [u8]) {
    unsafe {
        let p = std::ptr::slice_from_raw_parts(frame.pFrameData, frame.dataLen as usize)
            .as_ref()
            .unwrap();

        for (bgri, pi) in zip(bgr, p) {
            *bgri = *pi;
        }
    }
}

/// Checks if the number of pixels in `frame` equals `pixel_count`.
pub fn check_pixel_count(frame: &Frame, pixel_count: usize) {
    let w = frame.width as usize;
    let h = frame.height as usize;
    assert!(
        w * h == pixel_count,
        "pixel count is not equal to {} * {}",
        w,
        h
    );
}
