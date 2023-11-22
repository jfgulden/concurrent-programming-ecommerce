#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader},
        sync::{mpsc, Arc},
        thread,
        time::Duration,
    };

    use actix::{actors::mocker::Mocker, Actor, Recipient};
    use actix_rt::System;
    use concurrentes::{
        ecom::ecom_actor::EcomOrder,
        shop::{
            local_purchase::LocalPurchase,
            online_purchase::OnlinePurchase,
            shop_actor::{Product, Shop},
        },
        states::{LocalPurchaseState, OnlinePurchaseState},
    };
    use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

    #[actix_rt::test]
    async fn test_online_purchase_delivered_or_lost() {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("localhost:8765").unwrap();
            for a in listener.incoming() {
                let stream = a.unwrap();
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                tx.send(line).unwrap();
            }
        });

        let shop = Shop {
            name: "Tienda 1".to_string(),
            address: "".to_string(),
            location: 1,
            stock: vec![Product {
                id: "A".to_string(),
                stock: 10,
                reserved: 0,
            }],
        }
        .start();

        thread::sleep(Duration::from_millis(100));

        let stream = std::net::TcpStream::connect("localhost:8765").unwrap();
        let (_read, write) = tokio::io::split(TcpStream::from_std(stream).unwrap());

        let online_purchase = OnlinePurchase {
            id: 1,
            ecom: "1".to_string(),
            product: "A".to_string(),
            quantity: 5,
            state: OnlinePurchaseState::RECEIVED,
            zone_id: 1,
            write: Arc::new(Mutex::new(write)),
        };
        let middle_purchases = LocalPurchase {
            product: "A".to_string(),
            quantity: 2,
            status: LocalPurchaseState::CREATED,
        };

        let result = shop.send(online_purchase).await.unwrap().unwrap();

        assert_eq!(result, OnlinePurchaseState::RESERVED);
        let _ = shop.send(middle_purchases.clone()).await.unwrap();
        thread::sleep(Duration::from_millis(1100));
        let _ = shop.send(middle_purchases.clone()).await.unwrap();
        let _ = shop.send(middle_purchases.clone()).await.unwrap();
        let last_purchase = shop.send(middle_purchases.clone()).await.unwrap().unwrap();

        let mut received = rx.recv().unwrap();
        received.truncate(received.len() - 1);

        let order_str = received.split(',').collect::<Vec<&str>>();
        let state_number: u8 = order_str[1].parse().unwrap();
        let state = OnlinePurchaseState::from_int(state_number).unwrap();

        assert!(state == OnlinePurchaseState::DELIVERED || state == OnlinePurchaseState::LOST);
        if state == OnlinePurchaseState::DELIVERED {
            assert_eq!(last_purchase, LocalPurchaseState::REJECTED);
        } else {
            assert_eq!(last_purchase, LocalPurchaseState::SOLD);
        }
    }

    // #[actix_rt::test]
    // async fn test_connected_shop_receive_order() {
    //     let (tx, rx) = mpsc::channel();

    //     thread::spawn(|| {
    //         let system = System::new();
    //         system.block_on(async {
    //             let shop_mocker: Recipient<OnlinePurchase> =
    //                 Mocker::<OnlinePurchase>::mock(Box::new(move |msg, _ctx| {
    //                     let purchase = msg.downcast_ref::<OnlinePurchase>().unwrap().clone();
    //                     tx.send(purchase).unwrap();
    //                     msg
    //                 }))
    //                 .start()
    //                 .recipient();

    //             initiate_shop_server_side(shop_mocker, "localhost:8766".to_string())
    //                 .await
    //                 .unwrap();
    //         });
    //         system.run().unwrap();
    //     });

    //     thread::sleep(Duration::from_millis(1000));

    //     let stream = std::net::TcpStream::connect("localhost:8766").unwrap();
    //     let (_read, mut write) = tokio::io::split(TcpStream::from_std(stream).unwrap());

    //     let ecom_order = EcomOrder {
    //         id: 1,
    //         quantity: 1,
    //         product_id: "A".to_string(),
    //         shops_requested: vec![1],
    //         zone_id: 1,
    //     };

    //     write
    //         .write_all(ecom_order.to_string().as_bytes())
    //         .await
    //         .unwrap();

    //     let received = rx.recv().unwrap();
    //     assert!(received.id == 1);
    //     assert!(received.product == "A".to_string());
    //     assert!(received.quantity == 1);
    //     assert!(received.zone_id == 1);
    //     assert!(received.state == OnlinePurchaseState::RECEIVED);
    // }
}
