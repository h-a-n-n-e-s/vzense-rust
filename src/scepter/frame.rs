//! Reading frames, creating data arrays.

use std::iter::zip;

use super::device::Device;

use vzense_sys::scepter as sys;

/// Flag signaling if a frame is available
pub type FrameReady = sys::ScFrameReady;

/// Depth/IR/RGB image frame data.
pub type Frame = sys::ScFrame;

/// implement trait Data to allow use of type Frame in touch_detector
impl crate::util::touch_detector::Data for Frame {
    fn get_p_frame_data(&self) -> *mut u8 {
        self.pFrameData
    }
    fn get_data_len(&self) -> usize {
        self.dataLen as usize
    }
}

/// The available frame types, `Depth` and `RGB` (optical) and `IR` (infrared)
pub enum FrameType {
    /// depth
    Depth,
    /// optical
    RGB,
    /// optical mapped to the depth frame
    RGBMapped,
    /// Infrared
    IR,
}

/// Captures the next image frame from `device`. This API must be invoked before capturing frame data using `get_frame()`. `max_wait_time_ms` is the maximum waiting time for the next frame in milliseconds. The recommended value is 2 * 1000 / fps. `frame_ready` is a pointer to a buffer storing the signal for the frame availability.
pub fn read_next_frame(device: &Device, max_wait_time_ms: u16, frame_ready: &mut FrameReady) {
    unsafe {
        sys::scGetFrameReady(device.handle, max_wait_time_ms, frame_ready);
    }
}

/// Returns the image data in `frame` for the current frame from `device`. Before invoking this API, invoke `read_next_frame()` to capture one image frame from the device. `frame_ready` is a pointer to a buffer storing the signal for the frame availability set in `read_next_frame()`. The image `frame_type` is either `FrameType::Depth` or `FrameType::RGB`.
pub fn get_frame(
    device: &Device,
    frame_ready: &FrameReady,
    frame_type: &FrameType,
    frame: &mut Frame,
) {
    unsafe {
        match frame_type {
            FrameType::Depth => {
                if frame_ready.depth() == 1 {
                    sys::scGetFrame(device.handle, sys::ScFrameType_SC_DEPTH_FRAME, frame);
                }
            }
            FrameType::IR => {
                if frame_ready.ir() == 1 {
                    sys::scGetFrame(device.handle, sys::ScFrameType_SC_IR_FRAME, frame);
                }
            }
            FrameType::RGB => {
                if frame_ready.color() == 1 {
                    sys::scGetFrame(device.handle, sys::ScFrameType_SC_COLOR_FRAME, frame);
                }
            }
            FrameType::RGBMapped => {
                if frame_ready.transformedColor() == 1 {
                    let status = sys::scGetFrame(
                        device.handle,
                        sys::ScFrameType_SC_TRANSFORM_COLOR_IMG_TO_DEPTH_SENSOR_FRAME,
                        frame,
                    );
                    if status != sys::ScStatus_SC_OK {
                        panic!("get_frame failded with status {}", status);
                    }
                }
            }
        }
        if frame.pFrameData.is_null() {
            panic!("frame pointer is NULL!");
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

/// Creates `normalized_ir` data array from `frame`.
pub fn get_normalized_ir(frame: &Frame, min_ir: u8, max_ir: u8, normalized_ir: &mut [u8]) {
    unsafe {
        let p = std::ptr::slice_from_raw_parts(frame.pFrameData, frame.dataLen as usize)
            .as_ref()
            .unwrap();

        for (nii, pi) in zip(normalized_ir, p.iter()) {
            // scale to u8
            *nii = ((pi - min_ir) as f32 * 255.0 / (max_ir - min_ir) as f32).floor() as u8;
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
