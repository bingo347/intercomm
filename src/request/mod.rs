//! # Request-Response

use crate::common::StaticTypeMap;
use tokio::sync::{
    mpsc::{error::SendError, Sender, UnboundedSender},
    oneshot,
};

mod listener;

pub use listener::*;

static CHANNELS: StaticTypeMap = StaticTypeMap::new();

/// A request
pub trait Request: Sized + 'static {
    /// The number of requests
    /// that can be sent without waiting for the listener
    ///
    /// Set to 0 for unlimited buffer
    const BUFFER_SIZE: usize;

    /// Request name in debug messages
    const DEBUG_NAME: &'static str;

    /// Payload data type that will be sended with this request
    type Payload: Send;

    /// Response data type that will be responded from listener
    type Response: Send;
}

/// This enumeration is the list of the possible error outcomes for the
/// [request](crate::request::request) fn
#[non_exhaustive]
pub enum RequestError<R: Request> {
    /// Has not listener of this request
    NotListened(R::Payload),
    /// Internal request channel is closed
    SendError(R::Payload),
    /// Internal response channel is closed
    NotResponded,
}

/// Sends a payload to the [Listener](crate::request::Listener)
pub async fn request<R: Request>(payload: R::Payload) -> Result<R::Response, RequestError<R>> {
    let id = id!(R);
    let channels = CHANNELS.read().await;
    let sender = match channels.get(&id) {
        Some(sender) => sender,
        None => return Err(RequestError::NotListened(payload)),
    };
    let (tx, rx) = oneshot::channel();
    let request_pair = RequestPair::<R> {
        payload,
        responder: tx,
    };
    if R::BUFFER_SIZE == 0 {
        let sender: &UnboundedSender<_> = unsafe { sender.get_ref() };
        sender.send(request_pair)?;
    } else {
        let sender: &Sender<_> = unsafe { sender.get_ref() };
        sender.send(request_pair).await?;
    }
    rx.await.map_err(|_| RequestError::NotResponded)
}

struct RequestPair<R: Request> {
    payload: R::Payload,
    responder: oneshot::Sender<R::Response>,
}

impl<R: Request> From<SendError<RequestPair<R>>> for RequestError<R> {
    fn from(e: SendError<RequestPair<R>>) -> Self {
        let SendError(RequestPair { payload, .. }) = e;
        RequestError::SendError(payload)
    }
}

impl<R: Request> std::fmt::Debug for RequestError<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::NotListened(_) => {
                write!(f, "RequestError in {}: NotListened", R::DEBUG_NAME)?;
            }
            RequestError::SendError(_) => {
                write!(f, "RequestError in {}: SendError", R::DEBUG_NAME)?;
            }
            RequestError::NotResponded => {
                write!(f, "RequestError in {}: NotResponded", R::DEBUG_NAME)?;
            }
        }
        Ok(())
    }
}
