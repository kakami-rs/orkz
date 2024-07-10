use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let (tx, mut rx1) = broadcast::channel(10);
    let mut rx2 = tx.subscribe();

    tx.send("hello").unwrap();

    tokio::spawn(async move {
        loop {
            let _rx1 = rx1.recv();
            let _rx2 = rx2.recv();
            tokio::select! {
                _ = _rx1 => {
                    println!("rx1 received");
                }
                _ = _rx2 => {
                    println!("rx2 received");
                }
            }
        }
    });
    // tokio::spawn(async move {
    //     loop {
    //         let _rx2 = rx2.recv();
    //         tokio::select! {
    //             _ = _rx2 => {
    //                 println!("rx2 received");
    //             }
    //         }
    //     }
    // });
    for _ in 0..3 {
        tx.send("world").unwrap();
    }


    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
}
