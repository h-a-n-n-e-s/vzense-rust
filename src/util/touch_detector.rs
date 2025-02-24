//! A simple touch detector based on depth data.

use std::iter::zip;

use super::new_fixed_vec;

/// To allow invocation of generic devices from different APIs.
pub trait Data {
    fn get_frame_p_frame_data(&self) -> *mut u8;
    fn get_frame_data_len(&self) -> usize;
    fn get_min_depth_mm(&self) -> u16;
    fn get_max_depth_mm(&self) -> u16;
    fn current_frame_is_depth(&self) -> bool;
}

/**
The touch detector uses depth data to calculate the difference between the current depth and an initially recorded baseline depth. If this difference is between `min_touch` and `max_touch` a touch is assumed.

* `min_depth` and `max_depth` are the measuring ranges of the depth camera in mm.
* `min_touch` is the minimum height in mm above the surface considered to be a touch. If this parameter is too small, noise will lead to a lot of false detections.
* `max_touch` is the maximum height in mm above the surface considered to be a touch.

First, an average baseline depth is computed using the first `baseline_sample_size` frames. The current depth is estimated by a moving average of `sample_size` frames.
*/
pub struct TouchDetector {
    min_depth: u16,
    max_depth: u16,
    min_touch: f32,
    max_touch: f32,
    pixel_count: usize,
    baseline_sample_size: usize,
    sample_size: usize,
    baseline_sample: usize,
    sample: usize,
    baseline_depth_sum: Vec<u32>,
    depth_sum: Vec<u32>,
    ring_buffer: Vec<u16>,
}
impl TouchDetector {
    /// Creates a new instance with the specified parameters. All length parameters are in mm.
    pub fn new<Device: Data>(
        device: &Device,
        min_touch: f32,
        max_touch: f32,
        baseline_sample_size: usize,
        sample_size: usize,
        pixel_count: usize,
    ) -> Self {
        Self {
            min_depth: device.get_min_depth_mm(),
            max_depth: device.get_max_depth_mm(),
            min_touch,
            max_touch,
            pixel_count,
            baseline_sample_size,
            sample_size,
            baseline_sample: 0,
            sample: 0,
            baseline_depth_sum: new_fixed_vec(pixel_count, 0u32),
            depth_sum: new_fixed_vec(pixel_count, 0u32),
            ring_buffer: new_fixed_vec(sample_size * pixel_count, 0u16),
        }
    }

    /// Processes a depth frame resulting in a `touch_signal` (255 for "touch", 0 otherwise) and a `distance` from the initially measured depth in mm.
    ///
    /// **Note**: This function does nothing if the current frame in device is not a depth frame. Call `get_depth_mm_u16_frame()` or `get_depth_scaled_u8_frame()` before calling `process()`.
    pub fn process<Device: Data>(
        &mut self,
        device: &Device,
        touch_signal: &mut [u8],
        distance: &mut [f32],
    ) {
        // check if current frame holds a depth frame
        if device.current_frame_is_depth() {
            unsafe {
                let p = match std::ptr::slice_from_raw_parts(
                    device.get_frame_p_frame_data(),
                    device.get_frame_data_len(),
                )
                .as_ref()
                {
                    Some(ptr) => ptr,
                    None => return,
                };

                for (i, pi) in p.chunks_exact(2).enumerate() {
                    // create one u16 from two consecutive u8 and clamp to measuring range
                    let depth_mm =
                        u16::from_le_bytes([pi[0], pi[1]]).clamp(self.min_depth, self.max_depth);

                    // create baseline by averaging over first baseline_sample_size frames
                    if self.baseline_sample < self.baseline_sample_size {
                        self.baseline_depth_sum[i] += depth_mm as u32;
                    }

                    // pixel index of current sample in ring buffer
                    let j = self.pixel_count * self.sample + i;

                    // subtract old depth value in ring buffer from depth sum
                    self.depth_sum[i] -= self.ring_buffer[j] as u32;

                    // set ring buffer to new depth value and add it to depth sum
                    self.ring_buffer[j] = depth_mm;
                    self.depth_sum[i] += depth_mm as u32;

                    let diff = self.baseline_depth_sum[i] as f32 / self.baseline_sample_size as f32
                        - self.depth_sum[i] as f32 / self.sample_size as f32;

                    touch_signal[i] = if self.min_touch < diff && diff < self.max_touch {
                        255
                    } else {
                        0
                    };

                    distance[i] = diff;
                }
            }
            self.sample = (self.sample + 1) % self.sample_size;
            if self.baseline_sample < self.baseline_sample_size {
                self.baseline_sample += 1;
            }
        }
    }

    /// A by-product, returning only the normalized moving average of the depth.
    pub fn get_normalized_average_depth(&self, average_depth: &mut [u8]) {
        for (adi, dsi) in zip(average_depth, self.depth_sum.as_slice()) {
            let d = *dsi as f32 / self.sample_size as f32;

            *adi = ((d - self.min_depth as f32) * 255.0 / (self.max_depth - self.min_depth) as f32)
                .floor() as u8;
        }
    }

    /// The baseline depth as the average of the first `baseline_sample_size` frames.
    pub fn get_baseline(&self) -> Vec<f32> {
        let mut base_line = new_fixed_vec(self.baseline_depth_sum.len(), 0.0);
        for (a, b) in zip(self.baseline_depth_sum.as_slice(), base_line.as_mut_slice()) {
            *b = *a as f32 / self.baseline_sample_size as f32;
        }
        base_line
    }
}
