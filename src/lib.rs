#![doc = include_str!("../README.md")]
// #![warn(missing_docs)]

pub mod dcam560;
pub mod scepter;
pub mod util;

/// For the Depth and IR frames, the resolution is fixed to 640x480 for all data modes. The color frame can be set to higher resolutions using `set_color_resolution()`, but the defaults is also 640x480.
pub const DEFAULT_RESOLUTION: Resolution = Resolution::new(640, 480);
pub const DEFAULT_PIXEL_COUNT: usize = DEFAULT_RESOLUTION.to_pixel_count();

/// The available frame types, `Depth`, `IR` (infrared), `Color`, and `ColorMapped`.
#[derive(PartialEq)]
pub enum FrameType {
    Depth,
    IR,
    Color,
    ColorMapped,
}

pub enum ColorFormat {
    Rgb,
    Bgr,
}

/// Possible RGB resolutions.
#[derive(PartialEq)]
pub enum ColorResolution {
    Res640x480,
    Res800x600,
    Res1600x1200,
}

#[derive(PartialEq)]
pub struct Resolution {
    width: u32,
    height: u32,
}
impl Resolution {
    pub const fn new(w: u32, h: u32) -> Self {
        Self {
            width: w,
            height: h,
        }
    }
    pub fn to_array(&self) -> [u32; 2] {
        [self.width, self.height]
    }
    pub fn to_tuple(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub const fn to_pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }
    pub const fn double(&self) -> Self {
        Self {
            width: 2 * self.width,
            height: 2 * self.height,
        }
    }
}

/// Possible depth ranges. Only used for dcam560.
pub enum DepthRange {
    Near,
    Mid,
    Far,
}
