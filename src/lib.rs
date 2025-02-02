#![doc = include_str!("../README.md")]
// #![warn(missing_docs)]

pub mod dcam560;
pub mod scepter;

pub mod util;

/// The default resolution is 640x480. For depth and IR frames there is only this resolution. The color frame can be set to higher resolutions using `set_color_resolution()`, but the defaults is also 640x480.
pub const DEFAULT_RESOLUTION: Resolution = Resolution::new(640, 480);

/// Total number of pixels for `DEFAULT_RESOLUTION`.
pub const DEFAULT_PIXEL_COUNT: usize = DEFAULT_RESOLUTION.to_pixel_count();

/// Choose RGB or BGR format.
pub enum ColorFormat {
    Rgb,
    Bgr,
}

/// Possible color resolutions.
#[derive(PartialEq)]
pub enum ColorResolution {
    Res640x480,
    Res800x600,
    Res1600x1200,
}

/// Frame resolution.
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
    pub const fn to_array(&self) -> [u32; 2] {
        [self.width, self.height]
    }
    pub const fn to_tuple(&self) -> (u32, u32) {
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

/// Possible depth measuring ranges. Only used for DCAM560.
pub enum DepthMeasuringRange {
    Near,
    Mid,
    Far,
}
