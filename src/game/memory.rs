use std::mem;
use std::slice;

pub struct MemoryArena {
    size: usize,
    used: usize,
    base_ptr: *mut u8,
}

impl MemoryArena {
    pub fn new(new_size: usize, base_ptr: *const u8) -> MemoryArena {
        MemoryArena {
            size: new_size,
            used: 0,
            base_ptr: base_ptr as *mut u8,
        }
    }

    // TODO: Think about clear to zero options
    pub fn push_struct<'b, T>(&mut self) -> &'b mut T {
        let size = mem::size_of::<T>();
        debug_assert!(self.used + size <= self.size);

        let result_ptr = unsafe { self.base_ptr.offset(self.used as isize) };
        self.used += size;

        unsafe { &mut *(result_ptr as *mut T) }
    }

    pub fn push_slice<'b, T>(&mut self, count: usize) -> &'b mut [T] {
        let mem_size = count * mem::size_of::<T>();
        debug_assert!(self.used + mem_size <= self.size);

        let result_ptr = unsafe { self.base_ptr.offset(self.used as isize) };
        self.used += mem_size;

        unsafe { slice::from_raw_parts_mut(result_ptr as *mut T, count) }
    }
}
