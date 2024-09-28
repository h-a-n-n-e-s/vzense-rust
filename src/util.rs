use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::JoinHandle};

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
        Self {
            pressed,
            thread
        }
    }

    pub fn key_was_pressed(&self) -> bool {
        self.pressed.load(Ordering::Relaxed)
    }

    pub fn join(self) {
        self.thread.join().unwrap();
    }
}