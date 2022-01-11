use super::{Broadcast, CHANNELS};
use crate::common::UntypedBox;
use tokio::sync::broadcast::{channel, error::RecvError, Receiver, Sender};

/// Broadcast notification subscription
pub struct Subscription<B: Broadcast> {
    receiver: Option<Receiver<B::Payload>>,
}

/// Subscribe to broadcast notification
pub async fn subscribe<B: Broadcast>() -> Subscription<B> {
    let id = id!(B);
    let mut channels = CHANNELS.write().await;
    let rx = if let Some(channel) = channels.get(&id) {
        let channel = unsafe { channel.get_ref::<Sender<B::Payload>>() };
        channel.subscribe()
    } else {
        let (tx, rx) = channel(B::BUFFER_SIZE);
        let channel = UntypedBox::new(tx);
        channels.insert(id, channel);
        rx
    };
    Subscription { receiver: Some(rx) }
}

impl<B: Broadcast> Subscription<B> {
    /// Receives the next value for this Subscription
    pub async fn recv(&mut self) -> B::Payload {
        let receiver = match &mut self.receiver {
            Some(receiver) => receiver,
            None => unreachable!(),
        };
        loop {
            match receiver.recv().await {
                Ok(payload) => return payload,
                Err(RecvError::Closed) => unreachable!(),
                Err(RecvError::Lagged(_)) => {}
            }
        }
    }

    /// Closes the subscription
    ///
    /// Closing the subscription with this method
    /// is preferable for performance reasons
    pub async fn close(mut self) {
        drop(self.receiver.take());
        let id = id!(B);
        let mut channels = CHANNELS.write().await;
        if let Some(channel) = channels.get(&id) {
            let channel = unsafe { channel.get_ref::<Sender<B::Payload>>() };
            if channel.receiver_count() == 0 {
                channels.remove(&id);
            }
        }
    }
}

impl<B: Broadcast> Drop for Subscription<B> {
    fn drop(&mut self) {
        if self.receiver.is_some() {
            CHANNELS.remove_when_possible(id!(B));
        }
    }
}
