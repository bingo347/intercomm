//! # Notifications

use crate::common::StaticTypeMap;
use tokio::sync::mpsc::{error::SendError, Sender, UnboundedSender};

mod subscription;

#[cfg(test)]
mod test;

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
pub enum NotifyError<N: Notification> {
    /// Has not subscription of this notification
    NotSubscribed(N::Payload),
    /// Internal notification channel is closed
    SendError(N::Payload),
}

/// Sends a payload to the [Subscription](crate::notification::Subscription)
pub async fn notify<N: Notification>(payload: N::Payload) -> Result<(), NotifyError<N>> {
    let id = id!(N);
    let channels = CHANNELS.read().await;
    let sender = match channels.get(&id) {
        Some(sender) => sender,
        None => return Err(NotifyError::NotSubscribed(payload)),
    };
    if N::BUFFER_SIZE == 0 {
        let sender: &UnboundedSender<_> = unsafe { sender.get_ref() };
        sender.send(payload)?
    } else {
        let sender: &Sender<_> = unsafe { sender.get_ref() };
        sender.send(payload).await?
    }
    Ok(())
}

impl<N: Notification> From<SendError<N::Payload>> for NotifyError<N> {
    fn from(e: SendError<N::Payload>) -> Self {
        NotifyError::SendError(e.0)
    }
}

impl<N: Notification> std::fmt::Debug for NotifyError<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotifyError::NotSubscribed(_) => {
                write!(f, "NotifyError in {}: NotSubscribed", N::DEBUG_NAME)?;
            }
            NotifyError::SendError(_) => {
                write!(f, "NotifyError in {}: SendError", N::DEBUG_NAME)?;
            }
        }
        Ok(())
    }
}
