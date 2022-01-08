//! # Broadcast notifications

use crate::common::StaticTypeMap;
use tokio::sync::broadcast::Sender;

mod subscription;

#[cfg(test)]
mod test;

pub use subscription::*;

static CHANNELS: StaticTypeMap = StaticTypeMap::new();

/// A multi-notifier, single-subscriber notification
pub trait Broadcast: Sized + 'static {
    /// The number of notifications
    /// that can be sent without waiting for the receiver
    ///
    /// It must be at least 1
    const BUFFER_SIZE: usize;

    /// Notification name in debug messages
    const DEBUG_NAME: &'static str;

    /// Payload data type that will be sended with this notification
    type Payload: Clone + Send;
}

/// Sends a payload to the [Subscription](crate::broadcast::Subscription)
pub async fn notify<B: Broadcast>(payload: B::Payload) {
    let id = id!(B);
    let channels = CHANNELS.read().await;
    let sender = match channels.get(&id) {
        Some(sender) => sender,
        None => return,
    };
    let sender = unsafe { sender.get_ref::<Sender<B::Payload>>() };
    let _ = sender.send(payload);
}
