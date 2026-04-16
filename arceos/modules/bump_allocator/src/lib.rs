#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    // Memory range
    start: usize,
    end: usize,
    // Bytes allocation position (forward)
    b_pos: usize,
    // Pages allocation position (backward)
    p_pos: usize,
    // Number of active byte allocations
    count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
        self.count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        // Merge with existing memory range
        if self.end == start {
            self.end += size;
            self.p_pos = self.end;
        } else if start + size == self.start {
            self.start = start;
            self.b_pos = self.start;
        } else {
            // Cannot merge, just extend end
            self.end = start + size;
            self.p_pos = self.end;
        }
        Ok(())
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(&mut self, layout: core::alloc::Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();

        // Align b_pos
        let aligned_pos = (self.b_pos + align - 1) & !(align - 1);

        // Check if we have enough space
        if aligned_pos + size > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        // Allocate
        self.b_pos = aligned_pos + size;
        self.count += 1;

        Ok(NonNull::new(aligned_pos as *mut u8).unwrap())
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: core::alloc::Layout) {
        let size = layout.size();
        let addr = pos.as_ptr() as usize;

        // If deallocating the last allocation, we can reclaim the space
        if addr + size == self.b_pos {
            self.b_pos = addr;
        }
        self.count -= 1;

        // If all allocations are freed, reset bytes area
        if self.count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        let size = num_pages * Self::PAGE_SIZE;
        let align = 1 << align_pow2;

        // Align p_pos backward
        let aligned_pos = self.p_pos & !(align - 1);

        // Check if we have enough space
        if aligned_pos - size < self.b_pos {
            return Err(AllocError::NoMemory);
        }

        // Allocate backward
        self.p_pos = aligned_pos - size;

        Ok(self.p_pos)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        // Pages are never freed in this allocator (as per documentation)
        // But we can try to reclaim if it's the last allocation
        let size = num_pages * Self::PAGE_SIZE;
        if pos + size == self.p_pos {
            self.p_pos = pos;
        }
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / Self::PAGE_SIZE
    }
}