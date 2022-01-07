//! # Notifications

use crate::common::StaticTypeMap;
use tokio::sync::mpsc::{error::SendError, Sender, UnboundedSender};

mod subscription;

pub use subscription::*;

static CHANNELS: StaticTypeMap = StaticTypeMap::new();

/// A multi-notifier, single-subscriber notification
pub trait Notification: Sized + 'static {
    /// The number of notifications
    /// that can be sent without waiting for the receiver
    ///
    /// Set to 0 for unlimited buffer
    const BUFFER_SIZE: usize;

    /// Notification name in debug messages
    const DEBUG_NAME: &'static str;

    /// Payload data type that will be sended with this notification
    type Payload: Send;
}

/// This enumeration is the list of the possible error outcomes for the
/// [notify](crate::notification::notify) fn
#[non_exhaustive]
#[derive(Debug)]
pub enum NotifyError<Payload> {
    /// Has not subscription of this notification
    NotSubscribed(Payload),
    /// Internal notification channel is closed
    SendError(Payload),
}

/// Sends a payload to the [Subscription](crate::notification::Subscription)
pub async fn notify<N: Notification>(payload: N::Payload) -> Result<(), NotifyError<N::Payload>> {
    let id = id!(N);
    let channels = CHANNELS.read().await;
    let sender = match channels.get(&id) {
        Some(sender) => sender,
        None => return Err(NotifyError::NotSubscribed(payload)),
    };
    if N::BUFFER_SIZE == 0 {
        let sender = unsafe { sender.get_ref::<UnboundedSender<N::Payload>>() };
        sender.send(payload)?
    } else {
        let sender = unsafe { sender.get_ref::<Sender<N::Payload>>() };
        sender.send(payload).await?
    }
    Ok(())
}

impl<Payload> From<SendError<Payload>> for NotifyError<Payload> {
    fn from(e: SendError<Payload>) -> Self {
        NotifyError::SendError(e.0)
    }
}
