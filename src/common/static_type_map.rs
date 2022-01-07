use std::{any::TypeId, collections::HashMap};
use tokio::sync::{OnceCell, RwLock, RwLockReadGuard, RwLockWriteGuard};

type Map<V> = HashMap<TypeId, V>;

pub(crate) struct StaticTypeMap<V = super::UntypedBox> {
    inner: OnceCell<RwLock<Map<V>>>,
}

impl<V> StaticTypeMap<V> {
    pub(crate) const fn new() -> Self {
        Self {
            inner: OnceCell::const_new(),
        }
    }

    pub(crate) async fn read(&self) -> RwLockReadGuard<'_, Map<V>> {
        self.get_or_init().await.read().await
    }

    pub(crate) async fn write(&self) -> RwLockWriteGuard<'_, Map<V>> {
        self.get_or_init().await.write().await
    }

    async fn get_or_init(&self) -> &RwLock<Map<V>> {
        self.inner
            .get_or_init(|| async { RwLock::new(Map::new()) })
            .await
    }
}
