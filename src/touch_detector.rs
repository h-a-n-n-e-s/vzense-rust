use crate::util::new_fixed_vec;

/**
### A touch detector based on depth data.
* `min_depth` and `max_depth` are the measuring ranges of the depth camera.
* `min_touch` is the minimum height in mm above the surface considered to be a touch. If this parameter is too small, noise will lead to a lot of false detections.
* `max_touch` is the maximum height in mm above the surface considere to be a touch.
* `pix_count` is the total number of pixels in one frame.
* First an average baseline depth is computed using the first `baseline_sample_size` frames. The current depth is estimated by a moving average of `sample_size` frames.
*/
pub struct TouchDetector {
    min_depth: u16,
    max_depth: u16,
    min_touch: f32,
    max_touch: f32,
    pix_count: usize,
    pub baseline_sample_size: usize,
    sample_size: usize,
    sample: usize,
    baseline_depth_sum: Vec<u32>,
    depth_sum: Vec<u32>,
    ring_buffer: Vec<u16>,
}
impl TouchDetector {
    /// Create a new instance with the specified parameters.
    pub fn new(
        min_depth: u16,
        max_depth: u16,
        min_touch: f32,
        max_touch: f32,
        pix_count: usize,
        baseline_sample_size: usize,
        sample_size: usize,
    ) -> Self {
        Self {
            min_depth,
            max_depth,
            min_touch,
            max_touch,
            pix_count,
            baseline_sample_size,
            sample_size,
            sample: 0,
            baseline_depth_sum: new_fixed_vec(pix_count, 0u32),
            depth_sum: new_fixed_vec(pix_count, 0u32),
            ring_buffer: new_fixed_vec(sample_size * pix_count, 0u16),
        }
    }

    /// Processing one input `frame` resulting in a touch signal (255 for 'touch', 0 otherwise) stored in a \[u8] ready to be visualized. `count` should be a counter from the main animation loop (only necessary to know when to stop summing up the baseline).
    pub fn process_frame(
        &mut self,
        frame: &vzense_sys::PsFrame,
        count: usize,
        touch_signal: &mut [u8],
    ) {
        unsafe {
            let p = std::ptr::slice_from_raw_parts(frame.pFrameData, frame.dataLen as usize)
                .as_ref()
                .unwrap();

            for (i, v) in p.chunks_exact(2).enumerate() {
                // create one u16 from two consecutive u8 and clamp to measuring range
                let depth_mm =
                    u16::from_le_bytes([v[0], v[1]]).clamp(self.min_depth, self.max_depth);

                // create baseline by averaging over first baseline_sample_size frames
                if count < self.baseline_sample_size {
                    self.baseline_depth_sum[i] += depth_mm as u32;
                }

                // pixel index of current sample in ring buffer
                let j = self.pix_count * self.sample + i;

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
    }
}
