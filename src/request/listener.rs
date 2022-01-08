use super::{Request, RequestPair, CHANNELS};
use crate::common::UntypedBox;
use std::{mem, thread};
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver, UnboundedReceiver};

/// Request listener
pub struct Listener<R: Request> {
    receiver: RequestReceiver<R>,
}

/// Listen to request
///
/// Returns None if request is already listened
pub async fn listen<R: Request>() -> Option<Listener<R>> {
    let id = id!(R);
    if CHANNELS.read().await.contains_key(&id) {
        return None;
    }
    let (sender, receiver) = if R::BUFFER_SIZE == 0 {
        let (tx, rx) = unbounded_channel();
        let tx = UntypedBox::new(tx);
        let rx = RequestReceiver::Unbounded(rx);
        (tx, rx)
    } else {
        let (tx, rx) = channel(R::BUFFER_SIZE);
        let tx = UntypedBox::new(tx);
        let rx = RequestReceiver::Bounded(rx);
        (tx, rx)
    };
    CHANNELS.write().await.insert(id, sender);
    Some(Listener { receiver })
}

enum RequestReceiver<R: Request> {
    Bounded(Receiver<RequestPair<R>>),
    Unbounded(UnboundedReceiver<RequestPair<R>>),
    Closed,
}

impl<R: Request> Listener<R> {
    /// Accepts next request for this Listener
    pub async fn accept<F, Fut>(&mut self, f: F)
    where
        F: FnOnce(R::Payload) -> Fut,
        Fut: std::future::Future<Output = R::Response>,
    {
        let request_pair = match &mut self.receiver {
            RequestReceiver::Bounded(rx) => rx.recv().await,
            RequestReceiver::Unbounded(rx) => rx.recv().await,
            _ => unreachable!(),
        };
        let request_pair = match request_pair {
            Some(request_pair) => request_pair,
            None => unreachable!(),
        };
        let response = f(request_pair.payload).await;
        let _ = request_pair.responder.send(response);
    }

    /// Closes the listener
    ///
    /// Listeners must be closed with this method.
    /// Without it drop will panic
    pub async fn close(mut self) {
        let receiver = mem::replace(&mut self.receiver, RequestReceiver::Closed);
        match receiver {
            RequestReceiver::Bounded(mut rx) => rx.close(),
            RequestReceiver::Unbounded(mut rx) => rx.close(),
            _ => unreachable!(),
        }
        CHANNELS.write().await.remove(&id!(R));
    }
}

impl<R: Request> Drop for Listener<R> {
    fn drop(&mut self) {
        match &mut self.receiver {
            RequestReceiver::Bounded(rx) => rx.close(),
            RequestReceiver::Unbounded(rx) => rx.close(),
            RequestReceiver::Closed => return,
        }
        if !thread::panicking() {
            panic!("Abnormal listener close for {}", R::DEBUG_NAME);
        }
    }
}
