use rustix::mm::{
    mmap_anonymous, mprotect, munmap, MapFlags, MprotectFlags, ProtFlags,
};
use std::io::{Error, Result as IoResult};
use std::ptr::null_mut;

fn mmap_options(_stack: bool) -> MapFlags {
    let base = MapFlags::PRIVATE;

    // For FreeBSD we _shouldn't_ use MAP_STACK, as this inserts an implicit
    // guard page at the start of the returned pointer, and this could mess up
    // whatever expectations the user of the MemoryMap has. For example, for
    // Inko stacks the first page is private data and thus must be readable and
    // writable.
    //
    // OpenBSD doesn't have this behaviour, and on Linux MAP_STACK is a no-op.
    #[cfg(any(target_os = "linux", target_os = "openbsd"))]
    if _stack {
        return base | MapFlags::STACK;
    }

    base
}

/// A chunk of memory created using `mmap` and similar functions.
pub(crate) struct MemoryMap {
    pub(crate) ptr: *mut u8,
    pub(crate) len: usize,
}

impl MemoryMap {
    /// Allocates a new memory mapping suitable for use as stack memory.
    ///
    /// This method expects that `size` is a multiple of the page size. The
    /// alignment of the memory mapping is equal to its size.
    pub(crate) fn stack(size: usize) -> MemoryMap {
        // In order to align the desired region to its size, we have to allocate
        // more and manually align the resulting pointer.
        let alloc_size = size * 2;
        let opts = mmap_options(true);
        let res = unsafe {
            mmap_anonymous(
                null_mut(),
                alloc_size,
                ProtFlags::READ | ProtFlags::WRITE,
                opts,
            )
        };

        let ptr = match res {
            Ok(ptr) => ptr as *mut u8,
            Err(e) => panic!(
                "mmap(2) failed: {}",
                Error::from_raw_os_error(e.raw_os_error())
            ),
        };

        let start = ((ptr as usize + (size - 1)) & !(size - 1)) as *mut u8;
        let end = start as usize + size;

        // Due to the alignment we may end up with unused pages before or after
        // the aligned region. This ensures we get unmap those pages instead of
        // keeping them around while never using them.
        let unused_before = start as usize - ptr as usize;
        let unused_after = (ptr as usize + alloc_size) - end;

        unsafe {
            if unused_before > 0 {
                let _ = munmap(ptr as _, unused_before);
            }

            if unused_after > 0 {
                let _ = munmap(end as _, unused_after);
            }
        }

        MemoryMap { ptr: start, len: size }
    }

    pub(crate) fn protect(
        &mut self,
        start: usize,
        size: usize,
    ) -> IoResult<()> {
        let res = unsafe {
            mprotect(self.ptr.add(start) as _, size, MprotectFlags::empty())
        };

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::from_raw_os_error(e.raw_os_error())),
        }
    }
}

impl Drop for MemoryMap {
    fn drop(&mut self) {
        unsafe {
            let _ = munmap(self.ptr as _, self.len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustix::param::page_size;

    #[test]
    fn test_new() {
        let map1 = MemoryMap::stack(page_size());
        let map2 = MemoryMap::stack(page_size() * 3);

        assert_eq!(map1.len, page_size());
        assert_eq!(map2.len, page_size() * 3);
    }

    #[test]
    fn test_protect() {
        let mut map = MemoryMap::stack(page_size() * 2);

        assert!(map.protect(0, page_size()).is_ok());
    }
}
