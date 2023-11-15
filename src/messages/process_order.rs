use crate::error::PurchaseError;
use actix::Message;

#[derive(Debug, Message)]
#[rtype(result = "Result<(), PurchaseError>")]
pub struct ProcessOrders();
