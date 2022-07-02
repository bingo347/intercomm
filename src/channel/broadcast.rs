use tokio::sync::broadcast::channel;
pub(crate) use tokio::sync::broadcast::{Receiver, Sender};

impl<P: Send> super::Sender<P> for Sender<P> {}
impl<P: Send> super::Receiver<P> for Receiver<P> {}

pub(crate) fn factory<P: Clone, const BUFFER_SIZE: usize>() -> (Sender<P>, Receiver<P>) {
    channel(BUFFER_SIZE)
}
