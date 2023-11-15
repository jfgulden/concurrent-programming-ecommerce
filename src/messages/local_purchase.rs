use crate::error::PurchaseError;
use actix::Message;

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
