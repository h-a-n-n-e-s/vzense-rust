use std::iter::zip;

use crate::{device::DEFAULT_RESOLUTION, frame::Frame, util::new_fixed_vec};

/**
### A touch detector based on depth data.
* `min_depth` and `max_depth` are the measuring ranges of the depth camera.
* `min_touch` is the minimum height in mm above the surface considered to be a touch. If this parameter is too small, noise will lead to a lot of false detections.
* `max_touch` is the maximum height in mm above the surface considered to be a touch.
* First an average baseline depth is computed using the first `baseline_sample_size` frames. The current depth is estimated by a moving average of `sample_size` frames.
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
    /// Create a new instance with the specified parameters. All length parameters are in mm.
    pub fn new(
        min_depth: u16,
        max_depth: u16,
        min_touch: f32,
        max_touch: f32,
        baseline_sample_size: usize,
        sample_size: usize,
    ) -> Self {
        let pixel_count = DEFAULT_RESOLUTION.to_pixel_count();
        Self {
            min_depth,
            max_depth,
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

    /// Processing one input `frame` resulting in a touch signal (255 for 'touch', 0 otherwise) stored in `touch_signal`.
    pub fn process(&mut self, frame: &Frame, touch_signal: &mut [u8]) {
        unsafe {
            let p = std::ptr::slice_from_raw_parts(frame.pFrameData, frame.dataLen as usize)
                .as_ref()
                .unwrap();

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
            }
        }
        self.sample = (self.sample + 1) % self.sample_size;
        if self.baseline_sample < self.baseline_sample_size {
            self.baseline_sample += 1;
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
}
