use super::{OnceCell, UntypedBox};
use parking_lot::Mutex;
use std::{
    any::TypeId,
    collections::HashMap,
    mem,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

type Map<V> = HashMap<TypeId, V>;

pub(crate) struct StaticTypeMap<V = UntypedBox> {
    inner: OnceCell<StaticTypeMapInner<V>>,
}

struct StaticTypeMapInner<V> {
    has_to_remove: AtomicBool,
    to_remove: Mutex<Vec<TypeId>>,
    map: RwLock<Map<V>>,
}

impl<V> StaticTypeMap<V> {
    pub(crate) const fn new() -> Self {
        Self {
            inner: OnceCell::new(),
        }
    }

    pub(crate) async fn read(&self) -> RwLockReadGuard<'_, Map<V>> {
        self.get_or_init().map.read().await
    }

    pub(crate) async fn write(&self) -> RwLockWriteGuard<'_, Map<V>> {
        let inner = self.get_or_init();
        let mut map = inner.map.write().await;
        if inner.has_to_remove.load(Ordering::Relaxed) {
            eprintln!("clear");
            let mut to_remove = inner.to_remove.lock();
            inner.has_to_remove.store(false, Ordering::Release);
            let to_remove = mem::replace(&mut *to_remove, Vec::new());
            for id in to_remove.into_iter() {
                map.remove(&id);
            }
        }
        map
    }

    pub(crate) fn remove_when_possible(&self, id: TypeId) {
        let inner = self.get_or_init();
        inner.to_remove.lock().push(id);
        inner.has_to_remove.store(true, Ordering::Release);
    }

    fn get_or_init(&self) -> &StaticTypeMapInner<V> {
        self.inner.get_or_init(|| StaticTypeMapInner {
            has_to_remove: AtomicBool::new(false),
            to_remove: Mutex::new(Vec::new()),
            map: RwLock::new(Map::new()),
        })
    }
}
