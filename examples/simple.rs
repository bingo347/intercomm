use intercomm::{broadcast, notification, request};

intercomm::declare! {
    request[4] Sum((i32, i32)) -> i32;
    request[4] Mul((i32, i32)) -> i32;

    notification[2] Ready(());
    broadcast[1] Close(());
}

async fn sum_listener() {
    let mut listener = request::listen::<Sum>().await.expect("Sum listen twice");
    let mut close = broadcast::subscribe::<Close>().await;
    Ready::notify(()).await.expect("Cannot send Ready");

    loop {
        tokio::select! {
            _ = close.recv() => {
                listener.close().await;
                close.close().await;
                return;
            }
            _ = listener.accept(|(a, b)| async move {
                println!("Sum requested with: ({}, {})", a, b);
                a + b
            }) => {}
        }
    }
}

async fn mul_listener() {
    let mut listener = request::listen::<Mul>().await.expect("Mul listen twice");
    let mut close = broadcast::subscribe::<Close>().await;
    Ready::notify(()).await.expect("Cannot send Ready");

    loop {
        tokio::select! {
            _ = close.recv() => {
                listener.close().await;
                close.close().await;
                return;
            }
            _ = listener.accept(|(a, b)| async move {
                println!("Mul requested with: ({}, {})", a, b);
                a * b
            }) => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let mut ready = notification::subscribe::<Ready>()
        .await
        .expect("Ready subscribed twice");
    let sum_join = tokio::spawn(sum_listener());
    let mul_join = tokio::spawn(mul_listener());
    for _ in 0..2 {
        ready.recv().await;
    }
    ready.close().await;

    let sum = Sum::request((5, 10)).await.expect("Cannot request Sum");
    println!("5 + 10 = {}", sum);

    let mul = Mul::request((5, 10)).await.expect("Cannot request Mul");
    println!("5 * 10 = {}", mul);

    Close::notify(()).await;
    sum_join.await.expect("sum_listener panicked");
    mul_join.await.expect("mul_listener panicked");
}
