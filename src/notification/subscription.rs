use super::{Notification, CHANNELS};
use crate::common::UntypedBox;
use std::mem;
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver, UnboundedReceiver};

/// Notification subscription
pub struct Subscription<N: Notification> {
    receiver: SubscriptionReceiver<N::Payload>,
}

/// Subscribe to notification
///
/// Returns None if notification is already subscribed
pub async fn subscribe<N: Notification>() -> Option<Subscription<N>> {
    let id = id!(N);
    let mut channels = CHANNELS.write().await;
    if channels.contains_key(&id) {
        return None;
    }
    let (sender, receiver) = if N::BUFFER_SIZE == 0 {
        let (tx, rx) = unbounded_channel();
        let tx = UntypedBox::new(tx);
        let rx = SubscriptionReceiver::Unbounded(rx);
        (tx, rx)
    } else {
        let (tx, rx) = channel(N::BUFFER_SIZE);
        let tx = UntypedBox::new(tx);
        let rx = SubscriptionReceiver::Bounded(rx);
        (tx, rx)
    };
    channels.insert(id, sender);
    Some(Subscription { receiver })
}

enum SubscriptionReceiver<Payload> {
    Bounded(Receiver<Payload>),
    Unbounded(UnboundedReceiver<Payload>),
    Closed,
}

impl<N: Notification> Subscription<N> {
    /// Receives the next value for this Subscription
    pub async fn recv(&mut self) -> N::Payload {
        let payload = match &mut self.receiver {
            SubscriptionReceiver::Bounded(rx) => rx.recv().await,
            SubscriptionReceiver::Unbounded(rx) => rx.recv().await,
            _ => unreachable!(),
        };
        match payload {
            Some(payload) => payload,
            None => unreachable!(),
        }
    }

    /// Closes the subscription
    ///
    /// Closing the subscription with this method
    /// is preferable for performance reasons
    pub async fn close(mut self) {
        let receiver = mem::replace(&mut self.receiver, SubscriptionReceiver::Closed);
        match receiver {
            SubscriptionReceiver::Bounded(mut rx) => rx.close(),
            SubscriptionReceiver::Unbounded(mut rx) => rx.close(),
            _ => unreachable!(),
        }
        CHANNELS.write().await.remove(&id!(N));
    }
}

impl<N: Notification> Drop for Subscription<N> {
    fn drop(&mut self) {
        match &mut self.receiver {
            SubscriptionReceiver::Bounded(rx) => rx.close(),
            SubscriptionReceiver::Unbounded(rx) => rx.close(),
            SubscriptionReceiver::Closed => return,
        }
        CHANNELS.remove_when_possible(id!(N));
    }
}
