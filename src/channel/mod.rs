#![allow(dead_code, unused_imports)]

pub(crate) mod broadcast;
pub(crate) mod mpsc;

mod registry;
pub(crate) use self::registry::get_channel;

pub trait Channel: Sized + 'static {
    type Payload: Send;
    type Sender: Sender<Self::Payload>;
    type Receiver: Receiver<Self::Payload>;

    const FACTORY: fn() -> (Self::Sender, Self::Receiver);
}

pub trait Sender<Payload>: Clone + Sync {}
pub trait Receiver<Payload>: Sync {}
