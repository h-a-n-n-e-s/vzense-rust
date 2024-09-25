/// Creates a new vector of length `size` with capacity set to `size` and initializes it with `init`
pub fn new_fixed_vec<T: Clone>(size: usize, init: T) -> Vec<T> {
    let mut v = Vec::<T>::with_capacity(size);
    v.resize(size, init);
    v
}
