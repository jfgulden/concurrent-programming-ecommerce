use std::{sync::Arc, thread, time::Duration};

use crate::{messages::deliver_purchase::DeliverPurchase, shop_actor::Shop};
use actix::{AsyncContext, Context, Handler, Message};
use rand::{thread_rng, Rng};
use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};

#[derive(Debug, Clone)]

//EcomPurchase va a tener 5 estados: PROCESSING, RESERVED, DELIVERED, REJECTED o LOST.

pub enum EcomPurchaseState {
    PROCESSING,
    RESERVED,
    DELIVERED,
    REJECTED,
    LOST,
}
impl EcomPurchaseState {
    pub fn to_string(&self) -> String {
        match self {
            EcomPurchaseState::PROCESSING => "PROCESANDO".to_string(),
            EcomPurchaseState::RESERVED => "RESERVADO".to_string(),
            EcomPurchaseState::REJECTED => "RECHAZADO".to_string(),
            EcomPurchaseState::DELIVERED => "ENTREGADO".to_string(),
            EcomPurchaseState::LOST => "PERDIDO".to_string(),
        }
    }
    pub fn set_delivery_status(&mut self) {
        let is_delivered = thread_rng().gen_bool(0.5);
        if is_delivered {
            *self = EcomPurchaseState::DELIVERED;
        } else {
            *self = EcomPurchaseState::LOST;
        }
    }
    pub fn from_int(int: u8) -> Option<EcomPurchaseState> {
        match int {
            0 => Some(EcomPurchaseState::PROCESSING),
            1 => Some(EcomPurchaseState::RESERVED),
            2 => Some(EcomPurchaseState::REJECTED),
            3 => Some(EcomPurchaseState::DELIVERED),
            4 => Some(EcomPurchaseState::LOST),
            _ => None,
        }
    }
    pub fn to_int(&self) -> u8 {
        match self {
            EcomPurchaseState::PROCESSING => 0,
            EcomPurchaseState::RESERVED => 1,
            EcomPurchaseState::REJECTED => 2,
            EcomPurchaseState::DELIVERED => 3,
            EcomPurchaseState::LOST => 4,
        }
    }
}
impl PartialEq for EcomPurchaseState {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
// Message
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]

pub struct EcomPurchase {
    pub id: u8,
    pub product: String,
    pub quantity: u32,
    pub zone_id: u8,
    pub write: Arc<Mutex<WriteHalf<TcpStream>>>,
    pub state: EcomPurchaseState,
}

impl EcomPurchase {
    pub fn print_status(&self) {
        println!(
            "[ECOMM]  {} {:>2} x {}",
            self.state.to_string(),
            self.quantity,
            self.product
        );
    }
}

impl Handler<EcomPurchase> for Shop {
    type Result = ();

    fn handle(&mut self, mut msg: EcomPurchase, ctx: &mut Context<Self>) -> Self::Result {
        thread::sleep(Duration::from_millis(200));
        let product = self.stock.iter_mut().find(|p| p.id == msg.product);
        match product {
            Some(product) => {
                if product.stock < msg.quantity {
                    msg.state = EcomPurchaseState::REJECTED;
                } else {
                    product.stock -= msg.quantity;
                    product.reserved += msg.quantity;
                    msg.state = EcomPurchaseState::RESERVED;
                }
            }
            None => {
                msg.state = EcomPurchaseState::REJECTED;
            }
        }
        msg.print_status();
        ctx.address()
            .try_send(DeliverPurchase { purchase: msg })
            .unwrap();
    }
}

impl EcomPurchase {
    pub fn parse(line: Vec<&str>, write_half: Arc<Mutex<WriteHalf<TcpStream>>>) -> EcomPurchase {
        let id = line[0].parse::<u8>().unwrap();
        let product_id = line[1].to_string();
        let quantity = line[2].parse::<u32>().unwrap();
        let zone_id = line[3].parse::<u8>().unwrap();
        EcomPurchase {
            id,
            product: product_id,
            quantity,
            zone_id,
            write: write_half,
            state: EcomPurchaseState::PROCESSING,
        }
    }
}
