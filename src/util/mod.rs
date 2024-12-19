pub mod color_map;
pub mod touch_detector;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

/// Possible RGB resolutions.
#[derive(PartialEq)]
pub enum RGBResolution {
    RGBRes640x480,
    RGBRes800x600,
    RGBRes1600x1200,
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

/// For the Depth and IR frames, the resolution is fixed to 640x480 for all data modes. The rgb frame can be set to higher resolutions using `set_rgb_resolution()`, but the defaults is also 640x480.
pub const DEFAULT_RESOLUTION: Resolution = Resolution::new(640, 480);
pub const DEFAULT_PIXEL_COUNT: usize = DEFAULT_RESOLUTION.to_pixel_count();

/// Creates a new vector of length `size` with capacity set to `size` and initializes it with `init`.
pub fn new_fixed_vec<T: Clone>(size: usize, init: T) -> Vec<T> {
    let mut v = Vec::<T>::with_capacity(size);
    v.resize(size, init);
    v
}

// pub fn get_type_name<T>(var: &T) -> &'static str {
//     std::any::type_name_of_val(var)
// }

/// Simple keybord event handler.
pub struct KeyboardEvent {
    pressed: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}
impl KeyboardEvent {
    /// Create a new Event for the keystroke `key`.
    pub fn new(key: &str) -> Self {
        let pressed = Arc::new(AtomicBool::new(false));
        let pressed_cl = pressed.clone();
        let key = String::from(key);
        let thread = std::thread::spawn(move || {
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            // println!("{}", input);
            if input == key {
                pressed_cl.store(true, Ordering::Relaxed);
            }
        });
        Self { pressed, thread }
    }

    /// Check if a key was pressed.
    pub fn key_was_pressed(&self) -> bool {
        self.pressed.load(Ordering::Relaxed)
    }

    /// Join with the main thread.
    pub fn join(self) {
        self.thread.join().unwrap();
    }
}
