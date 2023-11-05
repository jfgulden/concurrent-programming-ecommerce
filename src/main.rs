use actix::prelude::*;

#[derive(Debug)]
struct Shop {
    name: String,
    address: String,
    location: u32,
    stock: Vec<Product>,
}

impl Actor for Shop {
    type Context = Context<Self>;
}

#[derive(Debug)]
enum PurchaseError {
    OutOfStock,
    NotDelivered,
}

// Message
#[derive(Debug, Message)]
#[rtype(result = "Result<(), PurchaseError>")]
struct LocalPurchase {
    product_id: String,
    quantity: u32,
}

impl Handler<LocalPurchase> for Shop {
    type Result = Result<(), PurchaseError>;

    fn handle(&mut self, msg: LocalPurchase, _ctx: &mut Context<Self>) -> Self::Result {
        let product = self
            .stock
            .iter_mut()
            .find(|p| p.id == msg.product_id)
            .ok_or(PurchaseError::OutOfStock)?;

        if product.stock < msg.quantity {
            return Err(PurchaseError::OutOfStock);
        }

        product.stock -= msg.quantity;

        Ok(())
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct Print;

impl Handler<Print> for Shop {
    type Result = ();

    fn handle(&mut self, _msg: Print, _ctx: &mut Context<Self>) -> Self::Result {
        println!("{:?}", self);
    }
}

#[derive(Debug)]
struct Product {
    id: String,
    stock: u32,
    reserved: u32,
}

#[actix_rt::main]
async fn main() {
    let shop = Shop {
        name: "My Shop".to_string(),
        address: "123 Main St".to_string(),
        location: 1,
        stock: vec![
            Product {
                id: "manzana".to_string(),
                stock: 10,
                reserved: 0,
            },
            Product {
                id: "banana".to_string(),
                stock: 11,
                reserved: 0,
            },
        ],
    }
    .start();

    shop.send(Print).await.unwrap();

    let result = shop
        .send(LocalPurchase {
            product_id: "manzana".to_string(),
            quantity: 5,
        })
        .await
        .unwrap();

    println!("{:?}", result);
    shop.send(Print).await.unwrap();

    let result = shop
        .send(LocalPurchase {
            product_id: "manzana".to_string(),
            quantity: 3,
        })
        .await
        .unwrap();

    println!("{:?}", result);
    shop.send(Print).await.unwrap();

    let result = shop
        .send(LocalPurchase {
            product_id: "manzana".to_string(),
            quantity: 7,
        })
        .await
        .unwrap();

    println!("{:?}", result);
    shop.send(Print).await.unwrap();
}
