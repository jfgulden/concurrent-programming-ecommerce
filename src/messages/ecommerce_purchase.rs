use std::{sync::Arc, thread, time::Duration};

use crate::{messages::deliver_purchase::DeliverPurchase, shop_actor::Shop};
use actix::{AsyncContext, Context, Handler, Message};
use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};

// Message
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]

pub struct EcommercePurchase {
    pub id: u8,
    pub product_id: String,
    pub quantity: u32,
    pub zone_id: u8,
    pub write: Arc<Mutex<WriteHalf<TcpStream>>>,
}

impl EcommercePurchase {
    pub fn print_cancelled(&self) {
        println!(
            "[ECOMM]  Rechazado {:>2} x {}, no hay stock",
            self.quantity, self.product_id
        );
    }
}

impl Handler<EcommercePurchase> for Shop {
    type Result = ();

    fn handle(&mut self, msg: EcommercePurchase, ctx: &mut Context<Self>) -> Self::Result {
        thread::sleep(Duration::from_millis(100));
        let product = self
            .stock
            .iter_mut()
            .find(|p| p.id == msg.product_id)
            .unwrap();

        if product.stock < msg.quantity {
            msg.print_cancelled();
            // mandar mensaje de que no hay stock
            return;
        }

        if product.stock > msg.quantity {
            product.stock -= msg.quantity;
        }
        product.reserved += msg.quantity;
        println!("[ECOM]  Reserva   {:>2} x {}", msg.quantity, msg.product_id);

        ctx.address()
            .try_send(DeliverPurchase { purchase: msg })
            .unwrap();
    }
}
