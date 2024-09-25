use crate::{device::Device, SESSION_INDEX};

use vzense_sys as sys;

/// Flag signaling if a frame is available
pub type FrameReady = sys::PsFrameReady;

/// Depth/IR/RGB image frame data.
pub type Frame = sys::PsFrame;

pub enum FrameType {
    Depth,
    Optical,
}

/// Captures the next image frame from `device`. This API must be invoked before capturing frame data using `get_frame()`. `frame_ready` is a pointer to a buffer storing the signal for the frame availability.
pub fn read_next_frame(device: Device, frame_ready: &mut FrameReady) {
    unsafe {
        sys::Ps2_ReadNextFrame(device, SESSION_INDEX, frame_ready);
    }
}

/// Returns the image data in `frame` for the current frame from the device specified by `device`. Before invoking this API, invoke `read_next_frame()` to capture one image frame from the device. `frame_ready` is a pointer to a buffer storing the signal for the frame availability set in `read_next_frame()`. The image `frame_type` is either `FrameType::Depth` or `FrameType::Optical`.
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
                    sys::Ps2_GetFrame(
                        device,
                        SESSION_INDEX,
                        sys::PsFrameType_PsDepthFrame,
                        frame,
                    );
                }
            }
            FrameType::Optical => {
                if frame_ready.rgb() == 1 {
                    sys::Ps2_GetFrame(
                        device,
                        SESSION_INDEX,
                        sys::PsFrameType_PsRGBFrame,
                        frame,
                    );
                }
            }
        }
    }
}

/// Creates depth data array `out` from `frame`.
pub fn get_depth_mono(frame: &sys::PsFrame, min_depth: u16, max_depth: u16, out: &mut [u8]) {
    unsafe {
        let p = std::ptr::slice_from_raw_parts(frame.pFrameData, frame.dataLen as usize)
            .as_ref()
            .unwrap();

        for (i, v) in p.chunks_exact(2).enumerate() {
            // create one u16 from two consecutive u8 and clamp to measuring range
            let depth_mm = u16::from_le_bytes([v[0], v[1]]).clamp(min_depth, max_depth);

            // scale to u8
            out[i] = ((depth_mm - min_depth) as f32 * 255.0 / (max_depth - min_depth) as f32)
                .floor() as u8;
        }
    }
}

/// Creates optical data array `out` from `frame`.
pub fn get_optical_rgb(frame: &sys::PsFrame, out: &mut [u8]) {
    unsafe {
        let p = std::ptr::slice_from_raw_parts(frame.pFrameData, frame.dataLen as usize)
            .as_ref()
            .unwrap();

        for (i, v) in p.iter().enumerate() {
            out[i] = *v;
        }
    }
}
