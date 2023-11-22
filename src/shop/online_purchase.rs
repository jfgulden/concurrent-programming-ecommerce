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
#[rtype(result = "()")]
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
    type Result = ();

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

        // si fue rechazado, se envia el rechazo
        if msg.state == OnlinePurchaseState::REJECTED {
            msg.send_msg(ctx);
            return;
        }

        ctx.address().do_send(DeliverPurchase { purchase: msg });
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
        let product_id = line[1].to_string();
        let quantity = line[2]
            .parse::<u32>()
            .map_err(|_| StreamError::WrongFormat)?;
        let zone_id = line[3]
            .parse::<u8>()
            .map_err(|_| StreamError::WrongFormat)?;

        Ok(OnlinePurchase {
            id,
            ecom,
            product: product_id,
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
