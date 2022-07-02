use super::Channel;
use crate::common::OnceCell;
use parking_lot::RwLock;
use std::{
    any::TypeId,
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
    ptr,
    sync::atomic::AtomicPtr,
};

type ChannelPtr<C> = ptr::NonNull<(<C as Channel>::Sender, <C as Channel>::Receiver)>;
type ChannelPair<C> = (
    ptr::NonNull<<C as Channel>::Sender>,
    ptr::NonNull<<C as Channel>::Receiver>,
);

struct Registry {
    set: RwLock<HashSet<RegistryItem>>,
}

struct RegistryItem {
    id: TypeId,
    channel: ptr::NonNull<()>,
}

pub(crate) fn get_channel<C: Channel>() -> ChannelPair<C> {
    let id = TypeId::of::<C>();
    let registry = get_registry();

    macro_rules! try_get {
        ($set:ident) => {
            if let Some(item) = $set.get(&id) {
                return split_channel::<C>(item.channel.cast());
            }
        };
    }

    // fast get with shared access
    let set = registry.set.read();
    try_get!(set);
    drop(set);

    // slow get with unique access
    let mut set = registry.set.write();
    try_get!(set);

    // create & store
    let (sender, receiver) = (C::FACTORY)();
    let ptr = Box::into_raw(Box::new((sender, receiver)));
    let ptr = unsafe { ptr::NonNull::new_unchecked(ptr) };
    set.insert(RegistryItem {
        id,
        channel: ptr.cast(),
    });
    split_channel::<C>(ptr)
}

fn split_channel<C: Channel>(ptr: ChannelPtr<C>) -> ChannelPair<C> {
    unsafe {
        let (ref mut sender, ref mut receiver) = *ptr.as_ptr();
        (
            ptr::NonNull::new_unchecked(sender),
            ptr::NonNull::new_unchecked(receiver),
        )
    }
}

fn get_registry() -> &'static Registry {
    static REGISTRY: OnceCell<Registry> = OnceCell::new();

    REGISTRY.get_or_init(|| Registry {
        set: RwLock::new(HashSet::new()),
    })
}

impl PartialEq for RegistryItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for RegistryItem {}

impl Hash for RegistryItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id, state);
    }
}

impl Borrow<TypeId> for RegistryItem {
    fn borrow(&self) -> &TypeId {
        &self.id
    }
}
