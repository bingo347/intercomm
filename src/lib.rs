#![warn(missing_docs)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::missing_panics_doc)]

//! Asynchronous inter-component communication library
//!
//! ## Example
//!
//! ```rust
//! use intercomm::{broadcast, notification, request};
//!
//! intercomm::declare! {
//!     request Sum((i32, i32)) -> i32;
//!     request Mul((i32, i32)) -> i32;
//!
//!     notification[2] Ready(());
//!     broadcast[1] Close(());
//! }
//!
//! async fn sum_listener() {
//!     let mut listener = request::listen::<Sum>().await.expect("Sum listen twice");
//!     let mut close = broadcast::subscribe::<Close>().await;
//!     Ready::notify(()).await.expect("Cannot send Ready");
//!
//!     loop {
//!         tokio::select! {
//!             _ = close.recv() => {
//!                 listener.close().await;
//!                 close.close().await;
//!                 return;
//!             }
//!             _ = listener.accept(|(a, b)| async move {
//!                 println!("Sum requested with: ({}, {})", a, b);
//!                 a + b
//!             }) => {}
//!         }
//!     }
//! }
//!
//! async fn mul_listener() {
//!     let mut listener = request::listen::<Mul>().await.expect("Mul listen twice");
//!     let mut close = broadcast::subscribe::<Close>().await;
//!     Ready::notify(()).await.expect("Cannot send Ready");
//!
//!     loop {
//!         tokio::select! {
//!             _ = close.recv() => {
//!                 listener.close().await;
//!                 close.close().await;
//!                 return;
//!             }
//!             _ = listener.accept(|(a, b)| async move {
//!                 println!("Mul requested with: ({}, {})", a, b);
//!                 a * b
//!             }) => {}
//!         }
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut ready = notification::subscribe::<Ready>()
//!         .await
//!         .expect("Ready subscribed twice");
//!     let sum_join = tokio::spawn(sum_listener());
//!     let mul_join = tokio::spawn(mul_listener());
//!     for _ in 0..2 {
//!         ready.recv().await;
//!     }
//!     ready.close().await;
//!
//!     let sum = Sum::request((5, 10)).await.expect("Cannot request Sum");
//!     println!("5 + 10 = {}", sum);
//!
//!     let mul = Mul::request((5, 10)).await.expect("Cannot request Mul");
//!     println!("5 * 10 = {}", mul);
//!
//!     Close::notify(()).await;
//!     sum_join.await.expect("sum_listener panicked");
//!     mul_join.await.expect("mul_listener panicked");
//! }
//! ```

#[macro_use]
mod common;

pub mod broadcast;
pub mod notification;
pub mod request;

#[doc(hidden)]
pub use doc_comment::doc_comment as __doc_comment;

/// Declare types for
/// [Broadcast](crate::broadcast::Broadcast),
/// [Notification](crate::notification::Notification),
/// [Request](crate::request::Request)
///
/// ## Syntax
///
/// `<visibility>? broadcast[<buffer size>] <name>(<payload type>);` \
/// `<visibility>? notification[<buffer size>] <name>(<payload type>);` \
/// `<visibility>? request[<buffer size>] <name>(<payload type>) -> <response type>;` \
///
/// `<buffer size>` is optional for `notification` & `request` and required for `broadcast`
///
///
/// ## Example
///
/// ```rust
/// intercomm::declare! {
///    /// B1 broadcast
///    broadcast[4] B1(i32);
///    /// B2 broadcast
///    pub(crate) broadcast[8] B2(i32);
///    /// B3 broadcast
///    pub broadcast[16] B3(i32);
///
///    /// N1 notification
///    notification N1(i32);
///    /// N2 notification
///    pub(crate) notification[1] N2(i32);
///    /// N3 notification
///    pub notification[4] N3(i32);
///
///    /// R1 request
///    request R1((i32, i32)) -> i32;
///    /// R2 request
///    pub(crate) request[4] R2((i32, i32)) -> i32;
///    /// R3 request
///    pub request[4] R3((i32, i32)) -> i32;
/// }
/// ```
#[macro_export]
macro_rules! declare {
    () => {};

    (
        $(#[$attr:meta])*
        $v:vis broadcast [$buffer_size:expr] $name:ident ($payload:ty);
        $($next:tt)*
    ) => {
        $(#[$attr])*
        $v struct $name;

        impl $crate::broadcast::Broadcast for $name {
            type Payload = $payload;
            const BUFFER_SIZE: usize = $buffer_size;
            const DEBUG_NAME: &'static str = stringify!($name);
        }

        impl $name {
            $crate::__doc_comment! {
                concat!("Sends a payload to the Subscription for ", stringify!($name)),
                $v async fn notify(payload: $payload) {
                    $crate::broadcast::notify::<$name>(payload).await
                }
            }
        }

        $crate::declare!($($next)*);
    };

    (
        $(#[$attr:meta])*
        $v:vis notification $([$buffer_size:expr])? $name:ident ($payload:ty);
        $($next:tt)*
    ) => {
        $(#[$attr])*
        $v struct $name;

        impl $crate::notification::Notification for $name {
            type Payload = $payload;
            const BUFFER_SIZE: usize = $crate::declare!(@buffer-size $($buffer_size)?);
            const DEBUG_NAME: &'static str = stringify!($name);
        }

        impl $name {
            $crate::__doc_comment! {
                concat!("Sends a payload to the Subscription for ", stringify!($name)),
                $v async fn notify(payload: $payload) -> Result<(), $crate::notification::NotifyError<$name>> {
                    $crate::notification::notify::<$name>(payload).await
                }
            }
        }

        $crate::declare!($($next)*);
    };

    (
        $(#[$attr:meta])*
        $v:vis request $([$buffer_size:expr])? $name:ident ($payload:ty) -> $response:ty;
        $($next:tt)*
    ) => {
        $(#[$attr])*
        $v struct $name;

        impl $crate::request::Request for $name {
            type Payload = $payload;
            type Response = $response;
            const BUFFER_SIZE: usize = $crate::declare!(@buffer-size $($buffer_size)?);
            const DEBUG_NAME: &'static str = stringify!($name);
        }

        impl $name {
            $crate::__doc_comment! {
                concat!("Sends a payload to the Listener for ", stringify!($name)),
                $v async fn request(payload: $payload) -> Result<$response, $crate::request::RequestError<$name>> {
                    $crate::request::request::<$name>(payload).await
                }
            }
        }

        $crate::declare!($($next)*);
    };

    (@buffer-size) => { 0 };
    (@buffer-size $buffer_size:expr) => { $buffer_size };
}
