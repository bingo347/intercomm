use parking_lot::{const_mutex, Mutex};
use tokio::sync::mpsc::{
    channel, unbounded_channel, Receiver as BoundedReceiver, Sender as BoundedSender,
    UnboundedReceiver, UnboundedSender,
};

pub(crate) enum Sender<P> {
    Bounded(BoundedSender<P>),
    Unbounded(UnboundedSender<P>),
}

pub(crate) struct Receiver<P> {
    raw: Mutex<Option<RawReceiver<P>>>,
}

pub(crate) enum RawReceiver<P> {
    Bounded(BoundedReceiver<P>),
    Unbounded(UnboundedReceiver<P>),
}

pub(crate) fn factory<P, const BUFFER_SIZE: usize>() -> (Sender<P>, Receiver<P>) {
    if BUFFER_SIZE == 0 {
        let (sender, receiver) = unbounded_channel();
        let sender = Sender::Unbounded(sender);
        let receiver = RawReceiver::Unbounded(receiver);
        (sender, Receiver::new(receiver))
    } else {
        let (sender, receiver) = channel(BUFFER_SIZE);
        let sender = Sender::Bounded(sender);
        let receiver = RawReceiver::Bounded(receiver);
        (sender, Receiver::new(receiver))
    }
}

impl<P> Sender<P> {
    pub(crate) async fn send(&self, value: P) {
        let r = match self {
            Sender::Bounded(sender) => sender.send(value).await,
            Sender::Unbounded(sender) => sender.send(value),
        };
        if let Err(_) = r {
            panic!("Channel must be always opened");
        }
    }
}

impl<P> Receiver<P> {
    const fn new(raw: RawReceiver<P>) -> Self {
        Self {
            raw: const_mutex(Some(raw)),
        }
    }

    pub(crate) fn take(&self) -> Option<RawReceiver<P>> {
        let mut receiver = self.raw.lock();
        receiver.take()
    }

    pub(crate) fn restore(&self, raw: RawReceiver<P>) {
        let mut receiver = self.raw.lock();
        *receiver = Some(raw);
    }
}

impl<P> RawReceiver<P> {
    pub(crate) async fn recv(&mut self) -> P {
        let payload = match self {
            RawReceiver::Bounded(receiver) => receiver.recv().await,
            RawReceiver::Unbounded(receiver) => receiver.recv().await,
        };
        payload.expect("Channel must be always opened")
    }
}

impl<P: Send> super::Sender<P> for Sender<P> {}
impl<P: Send> super::Receiver<P> for Receiver<P> {}

impl<P> Clone for Sender<P> {
    fn clone(&self) -> Self {
        match self {
            Sender::Bounded(sender) => Sender::Bounded(sender.clone()),
            Sender::Unbounded(sender) => Sender::Unbounded(sender.clone()),
        }
    }
}
