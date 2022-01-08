use super::*;

struct Broadcast1;

impl Broadcast for Broadcast1 {
    type Payload = i32;
    const BUFFER_SIZE: usize = 8;
    const DEBUG_NAME: &'static str = "Broadcast1";
}

async fn subscription1(id: i32) {
    println!("Subscribe: subscription1({})", id);
    let mut subscription = subscribe::<Broadcast1>().await;
    let mut last = 0i32;
    let mut i = 0;
    loop {
        i += 1;
        println!("subscription1({}): recv() #{}", id, i);
        let data = subscription.recv().await;
        println!("subscription1({}): received: {}", id, data);
        if data == 0 {
            println!("subscription1({}): close()", id);
            subscription.close().await;
            return;
        }
        assert!(data > last);
        last = data;
    }
}

#[tokio::test]
async fn many_subscribers() {
    println!("many_subscribers: Start subscriptions");
    let s1 = tokio::spawn(subscription1(1));
    let s2 = tokio::spawn(subscription1(2));
    tokio::task::yield_now().await;

    for i in 1..10 {
        println!("many_subscribers: notify() #{}", i);
        notify::<Broadcast1>(i).await;
    }
    println!("many_subscribers: notify() :finalize");
    notify::<Broadcast1>(0).await;

    println!("many_subscribers: Join subscriptions");
    s1.await.unwrap();
    s2.await.unwrap();

    assert!(CHANNELS.read().await.get(&id!(Broadcast1)).is_none());
}
