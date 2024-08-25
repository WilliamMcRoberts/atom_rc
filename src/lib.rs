use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{self, AtomicUsize, Ordering};

pub struct AtomRcInner<T> {
    rc: AtomicUsize,
    data: T,
}

pub struct AtomRc<T> {
    ptr: NonNull<AtomRcInner<T>>,
    phantom: PhantomData<AtomRcInner<T>>,
}

impl<T> AtomRc<T> {
    pub fn new(data: T) -> AtomRc<T> {
        // We start the reference count at 1, as that first reference is the
        // current pointer.
        let boxed = Box::new(AtomRcInner {
            rc: AtomicUsize::new(1),
            data,
        });
        AtomRc {
            // It is okay to call `.unwrap()` here as we get a pointer from
            // `Box::into_raw` which is guaranteed to not be null.
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            phantom: PhantomData,
        }
    }
}

impl<T> Clone for AtomRc<T> {
    fn clone(&self) -> AtomRc<T> {
        let inner = unsafe { self.ptr.as_ref() };
        // Using a relaxed ordering is alright here as we don't need any atomic
        // synchronization here as we're not modifying or accessing the inner
        // data.
        let old_rc = inner.rc.fetch_add(1, Ordering::Relaxed);

        if old_rc >= isize::MAX as usize {
            std::process::abort();
        }

        Self {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<T> Drop for AtomRc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };
        if inner.rc.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        // This fence is needed to prevent reordering of the use and deletion
        // of the data.
        atomic::fence(Ordering::Acquire);
        // This is safe as we know we have the last pointer to the `ArcInner`
        // and that its pointer is valid.
        unsafe {
            let _ = Box::from_raw(self.ptr.as_ptr());
        }
    }
}

impl<T> Deref for AtomRc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let inner = unsafe { self.ptr.as_ref() };
        &inner.data
    }
}

unsafe impl<T: Sync + Send> Send for AtomRc<T> {}
unsafe impl<T: Sync + Send> Sync for AtomRc<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
