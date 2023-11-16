use crate::error::PurchaseError;
use actix::Message;

// Message
#[derive(Debug, Message, Clone)]
#[rtype(result = "Result<(), PurchaseError>")]
//#[rtype(result = "bool")]

pub struct EcommercePurchase {
    pub product_id: String,
    pub quantity: u32,
    pub zone_id: u8,
}

impl EcommercePurchase {
    // pub fn serialize(&self) -> Vec<u8> {
    //     let mut buffer: Vec<u8> = Vec::new();
    //     let mut product = self.product_id.as_bytes().to_vec();
    //     while product.len() < 16 {
    //         product.push(0);
    //     }
    //     buffer.extend_from_slice(product.as_slice());
    //     buffer.extend_from_slice(&self.quantity.to_be_bytes());
    //     buffer.extend_from_slice(&self.zone_id.to_be_bytes());
    //     buffer
    // }

    // pub fn parse(buffer: Vec<u8>) -> Self {
    //     let mut product_id: Vec<u8> = Vec::new();
    //     product_id.extend_from_slice(&buffer[0..16]);
    //     let product_id = String::from_utf8(product_id).unwrap();
    //     let quantity = u32::from_be_bytes(buffer[16..20].try_into().unwrap());
    //     let zone_id = u8::from_be_bytes(buffer[20..21].try_into().unwrap());
    //     Self {
    //         product_id,
    //         quantity,
    //         zone_id,
    //     }
    // }

    pub fn print_cancelled(&self) {
        println!(
            "[ECOMM]  Rechazado {:>2} x {}, no hay stock",
            self.quantity, self.product_id
        );
    }
}
