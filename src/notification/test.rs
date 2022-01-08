use super::*;

struct Notification1;
struct Notification2;
struct Notification3;

struct Payload {
    data: Box<i32>,
}

impl Notification for Notification1 {
    type Payload = (i32, bool);
    const BUFFER_SIZE: usize = 1;
    const DEBUG_NAME: &'static str = "Notification1";
}

impl Notification for Notification2 {
    type Payload = Payload;
    const BUFFER_SIZE: usize = 0;
    const DEBUG_NAME: &'static str = "Notification2";
}

impl Notification for Notification3 {
    type Payload = ();
    const BUFFER_SIZE: usize = 1;
    const DEBUG_NAME: &'static str = "Notification3";
}

async fn subscription1() {
    println!("Subscribe: subscription1");
    let mut subscription = subscribe::<Notification1>().await.unwrap();
    let mut counter = 0i32;
    loop {
        counter += 1;
        println!("subscription1: recv() #{}", counter);
        let (data, end) = subscription.recv().await;
        assert_eq!(data, counter);
        if end {
            println!("subscription1: close()");
            subscription.close().await;
            return;
        }
    }
}

async fn subscription2() {
    println!("Subscribe: subscription2");
    let mut subscription = subscribe::<Notification2>().await.unwrap();
    for i in 1..=3i32 {
        println!("subscription2: recv() #{}", i);
        let Payload { data } = subscription.recv().await;
        assert_eq!(*data, i);
    }
    println!("subscription2: close()");
    subscription.close().await;
}

async fn subscription3() {
    println!("Subscribe: subscription3");
    let mut subscription = subscribe::<Notification3>().await.unwrap();
    println!("subscription3: recv()");
    subscription.recv().await;
    println!("subscription3: close()");
    subscription.close().await;
}

#[tokio::test]
async fn parallel_notifications() {
    println!("parallel_notifications: Start subscriptions");
    let s1 = tokio::spawn(subscription1());
    let s2 = tokio::spawn(subscription2());
    tokio::task::yield_now().await;

    for i in 1..=3i32 {
        println!("parallel_notifications: notify() #{}", i);
        notify::<Notification1>((i, i == 3)).await.unwrap();
        notify::<Notification2>(Payload { data: Box::new(i) })
            .await
            .unwrap();
    }

    println!("parallel_notifications: Join subscriptions");
    s1.await.unwrap();
    s2.await.unwrap();
}

#[tokio::test]
async fn reopen_subscription() {
    println!("reopen_subscription: Start subscription #1");
    let s1 = tokio::spawn(subscription3());
    tokio::task::yield_now().await;

    println!("reopen_subscription: notify #1");
    notify::<Notification3>(()).await.unwrap();

    println!("reopen_subscription: Join subscription #1");
    s1.await.unwrap();

    println!("reopen_subscription: Start subscription #2");
    let s2 = tokio::spawn(subscription3());
    tokio::task::yield_now().await;

    println!("reopen_subscription: notify #2");
    notify::<Notification3>(()).await.unwrap();

    println!("reopen_subscription: Join subscription #2");
    s2.await.unwrap();
}
