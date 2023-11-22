use std::{thread, time::Duration};

use crate::{constants::PURCHASE_MILLIS, error::PurchaseError, states::LocalPurchaseState};
use actix::{Context, Handler, Message};

use super::shop_actor::Shop;

#[derive(Debug, Message, Clone)]
#[rtype(result = "Result<LocalPurchaseState, PurchaseError>")]
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
    type Result = Result<LocalPurchaseState, PurchaseError>;

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

        Ok(msg.status.clone())
    }
}

#[cfg(test)]
mod tests {

    use actix::Actor;

    use crate::shop::shop_actor::Product;

    use super::*;

    #[actix_rt::test]
    async fn test_local_purchase_sold() {
        let shop = Shop {
            name: "Tienda 1".to_string(),
            address: "localhost:9888".to_string(),
            location: 1,
            stock: vec![Product {
                id: "A".to_string(),
                stock: 10,
                reserved: 0,
            }],
        }
        .start();

        let order = LocalPurchase {
            product: "A".to_string(),
            quantity: 1,
            status: LocalPurchaseState::CREATED,
        };

        let result = shop.send(order).await.unwrap();

        assert_eq!(result.unwrap(), LocalPurchaseState::SOLD);
    }

    #[actix_rt::test]
    async fn test_local_purchase_no_stock() {
        let shop = Shop {
            name: "Tienda 1".to_string(),
            address: "localhost:9888".to_string(),
            location: 1,
            stock: vec![Product {
                id: "A".to_string(),
                stock: 10,
                reserved: 0,
            }],
        }
        .start();

        let order = LocalPurchase {
            product: "A".to_string(),
            quantity: 11,
            status: LocalPurchaseState::CREATED,
        };

        let result = shop.send(order).await.unwrap();

        assert_eq!(result.unwrap(), LocalPurchaseState::REJECTED);
    }

    #[actix_rt::test]
    async fn test_local_purchase_no_product() {
        let shop = Shop {
            name: "Tienda 1".to_string(),
            address: "localhost:9888".to_string(),
            location: 1,
            stock: vec![Product {
                id: "A".to_string(),
                stock: 10,
                reserved: 0,
            }],
        }
        .start();

        let order = LocalPurchase {
            product: "B".to_string(),
            quantity: 1,
            status: LocalPurchaseState::CREATED,
        };

        let result = shop.send(order).await.unwrap();

        assert_eq!(result.unwrap(), LocalPurchaseState::REJECTED);
    }

    #[actix_rt::test]
    async fn test_local_purchase_order_until_no_stock() {
        let shop = Shop {
            name: "Tienda 1".to_string(),
            address: "localhost:9888".to_string(),
            location: 1,
            stock: vec![Product {
                id: "A".to_string(),
                stock: 10,
                reserved: 0,
            }],
        }
        .start();

        let order1 = LocalPurchase {
            product: "A".to_string(),
            quantity: 4,
            status: LocalPurchaseState::CREATED,
        };

        let result = shop.send(order1).await.unwrap();

        assert_eq!(result.unwrap(), LocalPurchaseState::SOLD);

        let order2 = LocalPurchase {
            product: "A".to_string(),
            quantity: 4,
            status: LocalPurchaseState::CREATED,
        };

        let result = shop.send(order2).await.unwrap();

        assert_eq!(result.unwrap(), LocalPurchaseState::SOLD);

        let order3 = LocalPurchase {
            product: "A".to_string(),
            quantity: 4,
            status: LocalPurchaseState::CREATED,
        };

        let result = shop.send(order3).await.unwrap();

        assert_eq!(result.unwrap(), LocalPurchaseState::REJECTED);
    }
}
