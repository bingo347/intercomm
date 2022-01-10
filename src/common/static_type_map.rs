use super::{OnceCell, UntypedBox};
use std::{any::TypeId, collections::HashMap};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

type Map<V> = HashMap<TypeId, V>;

pub(crate) struct StaticTypeMap<V = UntypedBox> {
    inner: OnceCell<RwLock<Map<V>>>,
}

impl<V> StaticTypeMap<V> {
    pub(crate) const fn new() -> Self {
        Self {
            inner: OnceCell::new(),
        }
    }

    pub(crate) async fn read(&self) -> RwLockReadGuard<'_, Map<V>> {
        self.get_or_init().read().await
    }

    pub(crate) async fn write(&self) -> RwLockWriteGuard<'_, Map<V>> {
        self.get_or_init().write().await
    }

    fn get_or_init(&self) -> &RwLock<Map<V>> {
        self.inner.get_or_init(|| RwLock::new(Map::new()))
    }
}
