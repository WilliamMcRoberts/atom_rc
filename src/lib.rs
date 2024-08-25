use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::AtomicUsize;

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
