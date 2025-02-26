//! Common utilities used by all APIs.

pub mod color_map;
pub mod touch_detector;

use std::{
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::Instant,
};

/// Creates a new vector of length `size` with capacity set to `size` and initializes it with `init`.
pub fn new_fixed_vec<T: Clone>(size: usize, init: T) -> Vec<T> {
    let mut v = Vec::<T>::with_capacity(size);
    v.resize(size, init);
    v
}

/// Simple keybord event handler.
pub struct KeyboardEvent {
    pressed: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}
impl KeyboardEvent {
    /// Creates a new event for the keystroke `key`.
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

    /// Check if the key was pressed.
    pub fn key_was_pressed(&self) -> bool {
        self.pressed.load(Ordering::Relaxed)
    }

    /// Join with the main thread.
    pub fn join(self) {
        self.thread.join().unwrap();
    }
}

/// A counter to be used in the main loop to get fps and frame count info. The `print_fps_frame_count_info()` function will be called every `info_interval`th loop.
pub struct Counter {
    count: u64,
    now: Instant,
    info_interval: u64,
}
impl Counter {
    pub fn new(info_interval: u64) -> Self {
        Self {
            count: 0,
            now: Instant::now(),
            info_interval,
        }
    }

    pub fn fps_frame_count_info(&mut self) -> Option<String> {
        self.count += 1;
        if self.count % self.info_interval == 0 {
            let elapsed = self.now.elapsed().as_secs_f64();
            self.now = Instant::now();
            return Some(format!(
                "  fps: {:.1}  frame: {}\r",
                self.info_interval as f64 / elapsed,
                self.count
            ));
        }
        None
    }

    pub fn print_fps_frame_count_info(&mut self) {
        if let Some(s) = self.fps_frame_count_info() {
            print!("{}", s);
            std::io::stdout().flush().unwrap();
        }
    }
}

/// normalize `[u16]` vector to `[u8]` given `min` and `max` value.
pub fn normalize_u16_to_u8(input: &[u16], min: u16, max: u16, norm: &mut [u8]) {
    let d = (max - min) as f32;
    for (n, i) in std::iter::zip(norm, input) {
        *n = (((*i).clamp(min, max) - min) as f32 * 255.0 / d).floor() as u8;
    }
}
