use std::alloc::{self, Layout};
use std::ptr::NonNull;
use std::{mem, ptr};

/// An array with lazy-initialized data storage.
///
/// Implementation is heavily inspired by <https://doc.rust-lang.org/nomicon/vec/vec-alloc.html>.
pub struct LazyArray<T> {
    /// The size of the data structure
    size: usize,
    /// The actual data
    data: NonNull<T>,
    /// The number of valid entries in `data`
    num_valid: usize,
    /// The indexable array of pointers to the data
    data_indices: NonNull<usize>,
    /// The back-pointers to the index array to check validity
    reverse_indices: NonNull<usize>,
}

impl<T> LazyArray<T> {
    pub fn new(size: usize) -> Self {
        assert!(mem::size_of::<T>() != 0);

        LazyArray {
            size,
            data: alloc_memory(size),
            num_valid: 0,
            data_indices: alloc_memory(size),
            reverse_indices: alloc_memory(size),
        }
    }
}

impl<T> LazyArray<T> {
    /// Get the value at `index` or None if the index is out of bounds or uninitialized.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        // out of bounds?
        if index >= self.size {
            return None;
        }

        unsafe {
            // Safety: We did check that `index` is inside the bounds
            self.get_data_index(index)
        }
        .map(|data_index| unsafe {
            // Safety: data_index is valid according to get_data_index
            self.data.add(data_index).as_ref()
        })
    }

    /// If `index` is a valid and initialized index, return the value stored there.
    /// If `index` is a valid but uninitialized index, generate and write the default value there and return it.
    /// If `index` is outside bounds, return None.
    #[inline]
    pub fn get_or_default_with(&mut self, index: usize, default: fn() -> T) -> Option<&T> {
        // out of bounds?
        if index >= self.size {
            return None;
        }

        unsafe {
            // Safety: We did check that `index` is inside the bounds
            self.get_or_create_data_index_with(index, default)
        }
        .map(|data_index| unsafe {
            // Safety: data_index is valid according to get_or_create_data_index_with
            self.data.add(data_index).as_ref()
        })
    }

    /// If `index` is a valid and initialized index, return the value stored there.
    /// If `index` is a valid but uninitialized index, generate and write `T::default()` there and return it.
    /// If `index` is outside bounds, return None.
    #[inline]
    pub fn get_or_default(&mut self, index: usize) -> Option<&T>
    where
        T: Default,
    {
        self.get_or_default_with(index, T::default)
    }

    /// If `index` is a valid and initialized index, return the value stored there.
    /// If `index` is a valid but uninitialized index, generate and write `default` there and return it.
    /// If `index` is outside bounds, return None.
    #[inline]
    pub fn get_or(&mut self, index: usize, default: T) -> Option<&T> {
        // out of bounds?
        if index >= self.size {
            return None;
        }

        unsafe {
            // Safety: We did check that `index` is inside the bounds
            self.get_or_create_data_index(index, default)
        }
        .map(|data_index| unsafe {
            // Safety: data_index is valid according to get_or_create_data_index_with
            self.data.add(data_index).as_ref()
        })
    }

    /// Set the value at `index`, dropping the previous value if there is one.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn set(&mut self, index: usize, element: T) {
        assert!(
            index < self.size,
            "Index {index} is out of bounds for LazyArray with size {}",
            self.size
        );

        // Safety: index is inside the bounds
        match unsafe { self.get_data_index(index) } {
            Some(data_index) => unsafe {
                // Safety: data_index is valid according to get_data_index.
                let _ = mem::replace(self.data.add(data_index).as_mut(), element);
            },
            None => {
                unsafe {
                    // Safety: index is inside the bounds but not initialized
                    self.create_data_index(index, element);
                }
            }
        }
    }

    /// Returns the index in `self.data` for the given `real_index` or `None` if it has not been initialized.
    ///
    /// The caller has to guarantee that `real_index` is inside the bounds.
    #[inline]
    unsafe fn get_data_index(&self, real_index: usize) -> Option<usize> {
        // Safety: This precondition must be true and is to be guaranteed by the caller.
        debug_assert!(real_index < self.size);

        // could the data_index be valid?
        // Safety: real_index is inside bounds
        let data_index = unsafe { ptr::read(self.data_indices.as_ptr().add(real_index)) };
        if data_index >= self.num_valid {
            return None; // nope, data_index points outside the valid area
        }

        // does the reverse_index confirm the data_index?
        // Safety: real_index is inside bounds
        let reverse_index = unsafe { ptr::read(self.reverse_indices.as_ptr().add(data_index)) };
        if reverse_index == real_index {
            Some(data_index) // yes, this is the correct data for this index
        } else {
            None // nope, reverse_index shows that this data_index belongs to another real index
        }
    }

    /// Insert `value` at the given `real_index` in the data structure.
    ///
    /// The caller has to guarantee that `real_index` is inside the bounds.
    /// and that it has not yet been assigned a value.
    #[inline]
    unsafe fn create_data_index(&mut self, real_index: usize, value: T) -> usize {
        // Safety: These preconditions must always be true and are to be guaranteed by the caller.
        debug_assert!(real_index < self.size);
        debug_assert!(self.num_valid < self.size);

        let data_index = self.num_valid;
        // Safety: `data_index` and `real_index` are valid due to preconditions
        self.data.add(data_index).write(value);
        self.reverse_indices.add(data_index).write(real_index);
        self.data_indices.add(real_index).write(data_index);

        self.num_valid += 1;
        data_index
    }

    /// Get the index in `self.data` for the given `real_index`.
    /// If it has not been initialized yet, store `default()` there.
    ///
    /// The caller has to guarantee that `real_index` is inside the bounds.
    #[inline]
    unsafe fn get_or_create_data_index(&mut self, real_index: usize, default: T) -> Option<usize> {
        // Safety: This precondition must be true and is to be guaranteed by the caller.
        debug_assert!(real_index < self.size);

        // Is this index already valid?
        // Safety: index is inside bounds (precondition)
        if let Some(data_index) = self.get_data_index(real_index) {
            Some(data_index) // yes, already initialized!
        } else {
            // No, this is uninitialized so far. Let's change that.
            // Safety: index is inside bounds (precondition) and uninitialized (we just checked that).
            Some(self.create_data_index(real_index, default))
        }
    }

    /// Get the index in `self.data` for the given `real_index`.
    /// If it has not been initialized yet, store `default()` there.
    ///
    /// The caller has to guarantee that `real_index` is inside the bounds.
    #[inline]
    unsafe fn get_or_create_data_index_with(
        &mut self,
        real_index: usize,
        default: fn() -> T,
    ) -> Option<usize> {
        // Safety: This precondition must be true and is to be guaranteed by the caller.
        debug_assert!(real_index < self.size);

        // Is this index already valid?
        // Safety: index is inside bounds (precondition)
        if let Some(data_index) = self.get_data_index(real_index) {
            Some(data_index) // yes, already initialized!
        } else {
            // No, this is uninitialized so far. Let's change that.
            // Safety: index is inside bounds (precondition) and uninitialized (we just checked that).
            Some(self.create_data_index(real_index, default()))
        }
    }
}

impl<T> Drop for LazyArray<T> {
    fn drop(&mut self) {
        let data_ptr = self.data.as_ptr();
        while self.num_valid > 0 {
            self.num_valid -= 1;
            unsafe {
                ptr::drop_in_place::<T>(data_ptr.add(self.num_valid));
            }
        }
        dealloc_memory(self.data.as_ptr(), self.size);
        dealloc_memory(self.data_indices.as_ptr(), self.size);
        dealloc_memory(self.reverse_indices.as_ptr(), self.size);
    }
}

fn alloc_memory<T>(size: usize) -> NonNull<T> {
    let layout = Layout::array::<T>(size).unwrap();
    let ptr = unsafe { alloc::alloc(layout) };
    match NonNull::new(ptr as *mut T) {
        Some(p) => p,
        None => alloc::handle_alloc_error(layout),
    }
}

fn dealloc_memory<T>(ptr: *mut T, size: usize) {
    let layout = Layout::array::<T>(size).unwrap();
    unsafe {
        alloc::dealloc(ptr as *mut u8, layout);
    }
}

#[cfg(test)]
mod test {
    use testdrop::TestDrop;

    use super::LazyArray;

    #[test]
    fn test_get_uninitialized_returns_none() {
        let lazy_array = LazyArray::<u32>::new(5);
        assert_eq!(lazy_array.get(2), None);
    }

    #[test]
    fn test_get_out_of_bounds_returns_none() {
        let lazy_array = LazyArray::<u32>::new(5);
        assert_eq!(lazy_array.get(5), None);
    }

    #[test]
    fn test_set_uninitialized_get() {
        let mut lazy_array = LazyArray::<u32>::new(5);
        lazy_array.set(2, 42);
        assert_eq!(lazy_array.get(2), Some(&42));
    }

    #[test]
    fn test_set_initialized_get() {
        let mut lazy_array = LazyArray::<u32>::new(5);
        lazy_array.set(2, 21);
        lazy_array.set(2, 42);
        assert_eq!(lazy_array.get(2), Some(&42));
    }

    #[test]
    fn test_set_get_multiple() {
        let mut lazy_array = LazyArray::<u32>::new(5);
        lazy_array.set(2, 21);
        lazy_array.set(2, 42);
        lazy_array.set(0, 84);
        assert_eq!(lazy_array.get(2), Some(&42));
        assert_eq!(lazy_array.get(0), Some(&84));
        assert_eq!(lazy_array.get(1), None);
    }

    #[test]
    fn test_drops_items() {
        let td = TestDrop::new();
        let (id, item) = td.new_item();
        {
            let mut lazy_array = LazyArray::new(5);
            lazy_array.set(2, item);
        }
        td.assert_drop(id);
    }

    #[test]
    fn test_set_initialized_drops_old() {
        let td = TestDrop::new();
        let mut lazy_array = LazyArray::new(5);
        let (id, item) = td.new_item();
        let (_, item2) = td.new_item();
        lazy_array.set(2, item);
        lazy_array.set(2, item2);
        td.assert_drop(id);
    }

    #[test]
    fn test_get_or_default_uninitialized_returns_default() {
        let mut lazy_array = LazyArray::<u32>::new(5);
        assert_eq!(lazy_array.get_or_default(2), Some(&0));
    }

    #[test]
    fn test_get_or_default_out_of_bounds_returns_none() {
        let mut lazy_array = LazyArray::<u32>::new(5);
        assert_eq!(lazy_array.get_or_default(5), None);
    }

    #[test]
    fn test_get_or_default_with_uninitialized_returns_default() {
        let mut lazy_array = LazyArray::<u32>::new(5);
        assert_eq!(lazy_array.get_or_default_with(2, || 42), Some(&42));
    }

    #[test]
    fn test_get_or_default_with_out_of_bounds_returns_none() {
        let mut lazy_array = LazyArray::<u32>::new(5);
        assert_eq!(lazy_array.get_or_default_with(5, || 42), None);
    }

    #[test]
    fn test_get_or_uninitialized_returns_value() {
        let mut lazy_array = LazyArray::<u32>::new(5);
        assert_eq!(lazy_array.get_or(2, 42), Some(&42));
    }

    #[test]
    fn test_get_or_out_of_bounds_returns_none() {
        let mut lazy_array = LazyArray::<u32>::new(5);
        assert_eq!(lazy_array.get_or(5, 42), None);
    }
}
