use super::*;
use std::sync::Arc;
use tokio::sync::Notify;

struct SumRequest;

impl Request for SumRequest {
    type Payload = (i32, i32);
    type Response = i32;
    const BUFFER_SIZE: usize = 2;
    const DEBUG_NAME: &'static str = "SumRequest";
}

async fn sum_listener(request_count: usize, ready: Arc<Notify>) {
    println!("Listen: Sum");
    let mut listener = listen::<SumRequest>().await.unwrap();
    ready.notify_one();
    for i in 0..request_count {
        println!("Accept: Sum #{}", i);
        listener.accept(|(a, b)| async move { a + b }).await;
    }
    listener.close().await;
}

#[tokio::test]
async fn sum_request() {
    let ready = Arc::new(Notify::new());
    println!("sum_request: Start listener");
    let l = tokio::spawn(sum_listener(3, ready.clone()));
    ready.notified().await;

    println!("sum_request: Send 3 requests");
    assert_eq!(request::<SumRequest>((1, 2)).await.unwrap(), 3);
    assert_eq!(request::<SumRequest>((2, 3)).await.unwrap(), 5);
    assert_eq!(request::<SumRequest>((3, 4)).await.unwrap(), 7);

    println!("sum_request: Send request to closed listener");
    assert!(matches!(
        request::<SumRequest>((0, 0)).await,
        Err(RequestError::NotListened(_))
    ));

    println!("sum_request: Join listener");
    l.await.unwrap();
}
