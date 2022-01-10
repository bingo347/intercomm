use parking_lot::Once;
use std::{cell::UnsafeCell, mem::MaybeUninit};

pub(crate) struct OnceCell<T> {
    once: Once,
    cell: UnsafeCell<MaybeUninit<T>>,
}

// it is correct because OnceCell provide outside read-only reference
// inner mutability totally blocked all threads that want OnceCell ref
unsafe impl<T> Sync for OnceCell<T> {}

impl<T> OnceCell<T> {
    pub(crate) const fn new() -> Self {
        Self {
            once: Once::new(),
            cell: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub(crate) fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
        self.once.call_once(|| {
            unsafe { &mut *self.cell.get() }.write(f());
        });
        unsafe { (&*self.cell.get()).assume_init_ref() }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};

    static IS_ALREADY_INIT: AtomicBool = AtomicBool::new(false);

    struct TestInitOnce(i32);

    fn init() -> TestInitOnce {
        if IS_ALREADY_INIT.load(SeqCst) {
            panic!("Init twice");
        }
        IS_ALREADY_INIT.store(true, SeqCst);
        println!("init");
        TestInitOnce(100)
    }

    #[test]
    fn once_cell() {
        let cell = OnceCell::new();
        let v1 = cell.get_or_init(init);
        let v2 = cell.get_or_init(init);
        assert_eq!(v1.0, v2.0);
    }
}
