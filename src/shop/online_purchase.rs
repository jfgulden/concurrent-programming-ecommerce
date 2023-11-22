use std::{sync::Arc, thread, time::Duration};

use actix::{dev::ContextFutureSpawner, fut::wrap_future, AsyncContext, Context, Handler, Message};
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

use crate::{constants::PURCHASE_MILLIS, error::StreamError, states::OnlinePurchaseState};

use super::{deliver_purchase::DeliverPurchase, shop_actor::Shop};

// Message
#[derive(Debug, Message, Clone)]
#[rtype(result = "Result<OnlinePurchaseState, ()>")]
pub struct OnlinePurchase {
    pub id: u8,
    pub ecom: String,
    pub product: String,
    pub quantity: u32,
    pub zone_id: u8,
    pub write: Arc<Mutex<WriteHalf<TcpStream>>>,
    pub state: OnlinePurchaseState,
}

impl Handler<OnlinePurchase> for Shop {
    type Result = Result<OnlinePurchaseState, ()>;

    fn handle(&mut self, mut msg: OnlinePurchase, ctx: &mut Context<Self>) -> Self::Result {
        thread::sleep(Duration::from_millis(PURCHASE_MILLIS));
        let product = self.stock.iter_mut().find(|p| p.id == msg.product);

        match product {
            Some(product) => {
                if product.stock < msg.quantity {
                    msg.state = OnlinePurchaseState::REJECTED;
                } else {
                    product.stock -= msg.quantity;
                    product.reserved += msg.quantity;
                    msg.state = OnlinePurchaseState::RESERVED;
                }
            }
            None => {
                msg.state = OnlinePurchaseState::REJECTED;
            }
        }
        msg.print_status();

        let result = msg.state.clone();
        // si fue rechazado, se envia el rechazo
        if msg.state == OnlinePurchaseState::REJECTED {
            msg.send_msg(ctx);
            return Ok(result);
        }

        ctx.address().do_send(DeliverPurchase { purchase: msg });

        Ok(result)
    }
}

impl OnlinePurchase {
    pub fn parse(
        line: Vec<&str>,
        ecom: String,
        write_half: Arc<Mutex<WriteHalf<TcpStream>>>,
    ) -> Result<OnlinePurchase, StreamError> {
        let id = line[0]
            .parse::<u8>()
            .map_err(|_| StreamError::WrongFormat)?;

        let product = line[1].to_string();
        let quantity = line[2]
            .parse::<u32>()
            .map_err(|_| StreamError::WrongFormat)?;
        let zone_id = line[3]
            .parse::<u8>()
            .map_err(|_| StreamError::WrongFormat)?;

        Ok(OnlinePurchase {
            id,
            ecom,
            product,
            quantity,
            zone_id,
            write: write_half,
            state: OnlinePurchaseState::RECEIVED,
        })
    }

    pub fn print_status(&self) {
        println!(
            "[ECOM {}]  {} {:>2} x {}",
            self.ecom,
            self.state.string_to_print(),
            self.quantity,
            self.product
        );
    }

    pub fn send_msg(self, ctx: &mut Context<Shop>) {
        wrap_future::<_, Shop>(async move {
            let mut write = self.write.lock().await;
            let msg = format!("{},{}\n", self.id, self.state.to_int());
            if write.write_all(msg.as_bytes()).await.is_err() {
                println!("Error al enviar mensaje");
            }
        })
        .wait(ctx);
    }
}

#[cfg(test)]
mod tests {
    use actix::Actor;
    use tokio::io::split;

    use super::*;
    use crate::{ecom::ecom_actor::EcomOrder, shop::shop_actor::Product};

    #[actix_rt::test]
    async fn test_parsing_online_purchase() {
        let order = EcomOrder {
            id: 1,
            product_id: String::from("manzana"),
            quantity: 1,
            zone_id: 1,
            shops_requested: Vec::new(),
        };
        let mut line: String = order.parse();
        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:18500").unwrap();
            listener.accept().unwrap();
        });
        thread::sleep(Duration::from_millis(100));
        let stream = std::net::TcpStream::connect("127.0.0.1:18500").unwrap();
        let tokio_stream = TcpStream::from_std(stream).unwrap();
        let (_read, write) = split(tokio_stream);
        line.truncate(line.len() - 1);
        let message = line.split(',').collect::<Vec<&str>>();
        let purchase =
            OnlinePurchase::parse(message, "2".to_string(), Arc::new(Mutex::new(write))).unwrap();
        assert_eq!(order.id, purchase.id.into());
        assert_eq!(order.product_id, purchase.product);
        assert_eq!("2".to_string(), purchase.ecom);
        assert_eq!(order.quantity, purchase.quantity);
        assert_eq!(order.zone_id, purchase.zone_id.into());
    }

    #[actix_rt::test]
    async fn test_online_purchase_sold() {
        let shop = Shop {
            name: "Tienda 1".to_string(),
            address: "localhost:9888".to_string(),
            location: 1,
            stock: vec![Product {
                id: "A".to_string(),
                stock: 10,
                reserved: 0,
            }],
        }
        .start();

        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:28510").unwrap();
            listener.accept().unwrap();
        });
        thread::sleep(Duration::from_millis(100));
        let stream = std::net::TcpStream::connect("127.0.0.1:28510").unwrap();
        let tokio_stream = TcpStream::from_std(stream).unwrap();
        let (_read, write) = split(tokio_stream);

        let order = OnlinePurchase {
            id: 1,
            ecom: "1".to_string(),
            zone_id: 1,
            write: Arc::new(Mutex::new(write)),
            product: "A".to_string(),
            quantity: 1,
            state: OnlinePurchaseState::RECEIVED,
        };

        let result = shop.send(order).await.unwrap();

        assert_eq!(result.unwrap(), OnlinePurchaseState::RESERVED);
    }

    #[actix_rt::test]
    async fn test_online_purchase_no_stock() {
        let shop = Shop {
            name: "Tienda 1".to_string(),
            address: "localhost:9888".to_string(),
            location: 1,
            stock: vec![Product {
                id: "A".to_string(),
                stock: 10,
                reserved: 0,
            }],
        }
        .start();

        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:28501").unwrap();
            listener.accept().unwrap();
        });
        thread::sleep(Duration::from_millis(100));
        let stream = std::net::TcpStream::connect("127.0.0.1:28501").unwrap();
        let tokio_stream = TcpStream::from_std(stream).unwrap();
        let (_read, write) = split(tokio_stream);

        let order = OnlinePurchase {
            id: 1,
            zone_id: 1,
            ecom: "1".to_string(),
            write: Arc::new(Mutex::new(write)),
            product: "A".to_string(),
            quantity: 11,
            state: OnlinePurchaseState::RECEIVED,
        };

        let result = shop.send(order).await.unwrap();

        assert_eq!(result.unwrap(), OnlinePurchaseState::REJECTED);
    }

    #[actix_rt::test]
    async fn test_online_purchase_no_product() {
        let shop = Shop {
            name: "Tienda 1".to_string(),
            address: "localhost:9888".to_string(),
            location: 1,
            stock: vec![Product {
                id: "A".to_string(),
                stock: 10,
                reserved: 0,
            }],
        }
        .start();

        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:28502").unwrap();
            listener.accept().unwrap();
        });
        thread::sleep(Duration::from_millis(100));
        let stream = std::net::TcpStream::connect("127.0.0.1:28502").unwrap();
        let tokio_stream = TcpStream::from_std(stream).unwrap();
        let (_read, write) = split(tokio_stream);

        let order = OnlinePurchase {
            id: 1,
            zone_id: 1,
            ecom: "1".to_string(),
            write: Arc::new(Mutex::new(write)),
            product: "B".to_string(),
            quantity: 1,
            state: OnlinePurchaseState::RECEIVED,
        };

        let result = shop.send(order).await.unwrap();

        assert_eq!(result.unwrap(), OnlinePurchaseState::REJECTED);
    }

    #[actix_rt::test]
    async fn test_online_purchase_order_until_no_stock() {
        let shop = Shop {
            name: "Tienda 1".to_string(),
            address: "localhost:9888".to_string(),
            location: 1,
            stock: vec![Product {
                id: "A".to_string(),
                stock: 10,
                reserved: 0,
            }],
        }
        .start();

        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:28503").unwrap();
            listener.accept().unwrap();
        });
        thread::sleep(Duration::from_millis(100));
        let stream = std::net::TcpStream::connect("127.0.0.1:28503").unwrap();
        let tokio_stream = TcpStream::from_std(stream).unwrap();
        let (_read, write) = split(tokio_stream);
        let write = Arc::new(Mutex::new(write));

        let order = OnlinePurchase {
            id: 1,
            zone_id: 1,
            ecom: "1".to_string(),
            write: write.clone(),
            product: "A".to_string(),
            quantity: 4,
            state: OnlinePurchaseState::RECEIVED,
        };

        let result = shop.send(order).await.unwrap();
        assert_eq!(result.unwrap(), OnlinePurchaseState::RESERVED);

        let order2 = OnlinePurchase {
            id: 1,
            zone_id: 1,
            ecom: "1".to_string(),
            write: write.clone(),
            product: "A".to_string(),
            quantity: 4,
            state: OnlinePurchaseState::RECEIVED,
        };

        let result = shop.send(order2).await.unwrap();
        assert_eq!(result.unwrap(), OnlinePurchaseState::RESERVED);

        let order3 = OnlinePurchase {
            id: 1,
            zone_id: 1,
            ecom: "1".to_string(),
            write: write.clone(),
            product: "A".to_string(),
            quantity: 4,
            state: OnlinePurchaseState::RECEIVED,
        };

        let result = shop.send(order3).await.unwrap();
        assert_eq!(result.unwrap(), OnlinePurchaseState::REJECTED);
    }
}
