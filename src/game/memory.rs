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

    //TODO: Think about clear to zero options
    pub fn push_struct<T>(&mut self) -> &'static mut T {
        let size = mem::size_of::<T>();
        debug_assert!(self.used + size <= self.size);

        let result_ptr = unsafe { self.base_ptr.offset(self.used as isize) };
        self.used += size;

        unsafe { mem::transmute(result_ptr) }
    }

    //This function pushes one element more than count on there so if the
    //user wants an array with 10 elements he gets 0 to 10 inclusive
    #[allow(dead_code)]
    pub fn push_slice<T>(&mut self, count: usize) -> &'static mut [T] {
        let mem_size = count * mem::size_of::<T>();
        debug_assert!(self.used + mem_size <= self.size);

        let result_ptr = unsafe { self.base_ptr.offset(self.used as isize) };
        self.used += mem_size;
        
        unsafe { slice::from_raw_parts_mut(result_ptr as *mut T, count) }
    }
}
