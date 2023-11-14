use crate::error::PurchaseError;
use actix::Message;

// Message
#[derive(Debug, Message)]
#[rtype(result = "Result<(), PurchaseError>")]

pub struct EcommercePurchase {
    pub product_id: String,
    pub quantity: u32,
}

impl EcommercePurchase {
    pub fn print_cancelled(&self) {
        println!(
            "[ECOMM] Rechazado {:>2} x {}, no hay stock",
            self.quantity, self.product_id
        );
    }
}
