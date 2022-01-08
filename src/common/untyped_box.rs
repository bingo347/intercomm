use std::any::Any;

type Value = dyn Any + Send + Sync;

pub(crate) struct UntypedBox {
    inner: Box<Value>,
}

impl UntypedBox {
    pub(crate) fn new<T: Send + Sync + 'static>(value: T) -> Self {
        Self {
            inner: Box::new(value),
        }
    }

    /// Safety: T must be some type that used in UntypedBox::new
    pub(crate) unsafe fn consume<T>(self) -> Box<T> {
        let raw = Box::into_raw(self.inner);
        Box::from_raw(raw as *mut T)
    }

    /// Safety: T must be some type that used in UntypedBox::new
    pub(crate) unsafe fn get_ref<T>(&self) -> &T {
        &*(self.inner.as_ref() as *const Value as *const T)
    }

    /// Safety: T must be some type that used in UntypedBox::new
    pub(crate) unsafe fn get_mut<T>(&mut self) -> &mut T {
        &mut *(self.inner.as_mut() as *mut Value as *mut T)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn untyped_box() {
        let mut b = UntypedBox::new(0i32);
        assert_eq!(unsafe { *b.get_ref::<i32>() }, 0);

        unsafe {
            *b.get_mut::<i32>() = 1;
        }
        assert_eq!(unsafe { *b.get_ref::<i32>() }, 1);

        let b = unsafe { b.consume::<i32>() };
        assert_eq!(*b, 1);
    }
}
