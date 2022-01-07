use std::any::Any;

pub(crate) struct UntypedBox {
    inner: Box<dyn Any>,
}

impl UntypedBox {
    pub(crate) fn new<T: 'static>(value: T) -> Self {
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
        &*(self.inner.as_ref() as *const dyn Any as *const T)
    }

    /// Safety: T must be some type that used in UntypedBox::new
    pub(crate) unsafe fn get_mut<T>(&mut self) -> &mut T {
        &mut *(self.inner.as_mut() as *mut dyn Any as *mut T)
    }
}
