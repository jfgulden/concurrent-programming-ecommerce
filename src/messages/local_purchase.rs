use crate::{error::PurchaseError, shop_actor::Shop};
use actix::{Context, Handler, Message};

// Message
#[derive(Debug, Message)]
#[rtype(result = "Result<(), PurchaseError>")]

pub struct LocalPurchase {
    pub product_id: String,
    pub quantity: u32,
}

impl LocalPurchase {
    pub fn print_cancelled(&self) {
        println!(
            "[LOCAL]  Rechazado {:>2} x {}, no hay stock",
            self.quantity, self.product_id
        );
    }
}

impl Handler<LocalPurchase> for Shop {
    type Result = Result<(), PurchaseError>;

    fn handle(&mut self, msg: LocalPurchase, _ctx: &mut Context<Self>) -> Self::Result {
        // thread::sleep(Duration::from_millis(thread_rng().gen_range(500..1500)));

        let product = match self.stock.iter_mut().find(|p| p.id == msg.product_id) {
            Some(product) => product,
            None => {
                msg.print_cancelled();
                return Err(PurchaseError::OutOfStock);
            }
        };

        if product.stock < msg.quantity {
            msg.print_cancelled();
            return Err(PurchaseError::OutOfStock);
        }

        product.stock -= msg.quantity;

        println!(
            "[LOCAL]  Vendido   {:>2} x {}, quedan {}",
            msg.quantity, msg.product_id, product.stock
        );

        Ok(())
    }
}
