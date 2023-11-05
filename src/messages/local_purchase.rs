use crate::error::PurchaseError;
use actix::Message;

// Message
#[derive(Debug, Message)]
#[rtype(result = "Result<(), PurchaseError>")]

pub struct LocalPurchase {
    pub product_id: String,
    pub quantity: u32,
}
