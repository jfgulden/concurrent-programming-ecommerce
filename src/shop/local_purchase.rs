use std::{thread, time::Duration};

use crate::{constants::PURCHASE_MILLIS, error::PurchaseError, states::LocalPurchaseState};
use actix::{Context, Handler, Message};

use super::shop_actor::Shop;

#[derive(Debug, Message, Clone)]
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
        thread::sleep(Duration::from_millis(PURCHASE_MILLIS));

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
