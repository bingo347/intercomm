#![warn(missing_docs)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::missing_panics_doc)]

//! Asynchronous inter-component communication library

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
