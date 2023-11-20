use std::{thread, time::Duration};

use crate::{error::PurchaseError, shop_actor::Shop};
use actix::{Context, Handler, Message};
use rand::{thread_rng, Rng};

//LocalPurchase solo va a tener 3 estados: PROCESSING, SOLD o REJECTED.
#[derive(Debug, Clone)]
pub enum LocalPurchaseState {
    PROCESSING,
    SOLD,
    REJECTED,
}
impl LocalPurchaseState {
    pub fn to_string(&self) -> String {
        match self {
            LocalPurchaseState::PROCESSING => "PROCESANDO".to_string(),
            LocalPurchaseState::SOLD => "VENDIDO".to_string(),
            LocalPurchaseState::REJECTED => "RECHAZADO".to_string(),
        }
    }
}
#[derive(Debug, Message)]
#[rtype(result = "Result<(), PurchaseError>")]
pub struct LocalPurchase {
    pub product: String,
    pub quantity: u32,
    pub status: LocalPurchaseState,
}

impl LocalPurchase {
    pub fn print_status(&self) {
        println!(
            "[LOCAL]  {} {:>2} x {}",
            self.status.to_string(),
            self.quantity,
            self.product
        );
    }
}

impl Handler<LocalPurchase> for Shop {
    type Result = Result<(), PurchaseError>;

    fn handle(&mut self, mut msg: LocalPurchase, _ctx: &mut Context<Self>) -> Self::Result {
        thread::sleep(Duration::from_millis(thread_rng().gen_range(200..=200)));

        match self.stock.iter_mut().find(|p| p.id == msg.product) {
            Some(product) => {
                if product.stock < msg.quantity {
                    msg.status = LocalPurchaseState::REJECTED;
                } else {
                    msg.status = LocalPurchaseState::SOLD;
                    product.stock -= msg.quantity;
                }
            }
            None => {
                msg.status = LocalPurchaseState::REJECTED;
            }
        };
        msg.print_status();

        Ok(())
    }
}
