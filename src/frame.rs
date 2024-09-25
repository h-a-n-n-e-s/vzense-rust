use crate::SESSION_INDEX;

use vzense_sys as sys;

pub type FrameReady = sys::PsFrameReady;
pub type Frame = sys::PsFrame;

pub enum FrameType {
    Depth,
    Optical,
}

pub fn read_next_frame(device_handle: sys::PsDeviceHandle, frame_ready: &mut sys::PsFrameReady) {
    unsafe {
        sys::Ps2_ReadNextFrame(device_handle, SESSION_INDEX, frame_ready);
        // if status != ok {
        //     panic!("read next frame failed with status {}", status);
        // }
    }
}

pub fn get_frame(
    device_handle: sys::PsDeviceHandle,
    frame_ready: &sys::PsFrameReady,
    frame_type: FrameType,
    frame: &mut sys::PsFrame,
) {
    unsafe {
        match frame_type {
            FrameType::Depth => {
                if frame_ready.depth() == 1 {
                    sys::Ps2_GetFrame(
                        device_handle,
                        SESSION_INDEX,
                        sys::PsFrameType_PsDepthFrame,
                        frame,
                    );
                }
            }
            FrameType::Optical => {
                if frame_ready.rgb() == 1 {
                    sys::Ps2_GetFrame(
                        device_handle,
                        SESSION_INDEX,
                        sys::PsFrameType_PsRGBFrame,
                        frame,
                    );
                }
            }
        }
    }
}

pub fn get_depth_u8(frame: &sys::PsFrame, min_depth: u16, max_depth: u16, out: &mut [u8]) {
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
