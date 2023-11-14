use crate::error::{FileError, PurchaseError};
use crate::messages::ecommerce_purchase::EcommercePurchase;
use crate::messages::local_purchase::*;
use crate::messages::print::Print;
use actix::{Actor, ActorFutureExt, Context, Handler, ResponseActFuture, WrapFuture};
use actix_rt::time::sleep;
use rand::{thread_rng, Rng};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::time::Duration;

#[derive(Debug)]
pub struct Product {
    pub id: String,
    pub stock: u32,
    pub reserved: u32,
}

#[derive(Debug)]
pub struct Shop {
    pub name: String,
    pub address: String,
    pub location: u32,
    pub stock: Vec<Product>,
}

impl Shop {
    pub fn from_file(path: &str) -> Result<Self, FileError> {
        let file = File::open(path).map_err(|_| FileError::NotFound)?;
        let shop = Self::from_reader(file)?;
        println!("===");
        println!("Sucursal:  {}", shop.name);
        println!("Servidor:  {}", shop.address);
        println!("Sector:    {}", shop.location);
        println!("===");
        println!("Stock:");
        for product in &shop.stock {
            println!("  - {:<3} x {}", product.stock, product.id);
        }
        println!("===\n");
        Ok(shop)
    }

    fn from_reader<T: Read>(content: T) -> Result<Shop, FileError> {
        let reader = BufReader::new(content);

        let mut lines = reader.lines();

        let shop_info_string = match lines.next() {
            Some(string) => string.map_err(|_| FileError::WrongFormat)?,
            None => return Err(FileError::WrongFormat),
        };

        let shop_info: Vec<&str> = shop_info_string.split(',').collect();
        if shop_info.len() != 3 {
            return Err(FileError::WrongFormat);
        }

        let mut shop = Self {
            name: shop_info[0].to_string(),
            address: shop_info[1].to_string(),
            location: shop_info[2].parse().map_err(|_| FileError::WrongFormat)?,
            stock: Vec::new(),
        };

        // ignore dash line
        lines.next();

        for line in lines {
            let current_line = line.map_err(|_| FileError::WrongFormat)?;

            let product_data: Vec<&str> = current_line.split(',').collect();

            // ['KEY', 'VALUE'].len() == 2
            if product_data.len() != 2 {
                return Err(FileError::WrongFormat);
            }
            let product = Product {
                id: product_data[0].to_string(),
                stock: product_data[1]
                    .parse()
                    .map_err(|_| FileError::WrongFormat)?,
                reserved: 0,
            };

            shop.stock.push(product);
        }

        Ok(shop)
    }
}

impl Actor for Shop {
    type Context = Context<Self>;
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
            "[LOCAL] Vendido   {:>2} x {}, quedan {}",
            msg.quantity, msg.product_id, product.stock
        );

        Ok(())
    }
}

impl Handler<EcommercePurchase> for Shop {
    type Result = ResponseActFuture<Self, Result<(), PurchaseError>>;

    fn handle(&mut self, msg: EcommercePurchase, _ctx: &mut Context<Self>) -> Self::Result {
        // thread::sleep(Duration::from_millis(thread_rng().gen_range(500..1500)));

        println!("[ECOMM] Reserva   {:>2} x {}", msg.quantity, msg.product_id);

        let millis = thread_rng().gen_range(500..=500);
        Box::pin(sleep(Duration::from_millis(millis)).into_actor(self).map(
            move |_result, _me, _ctx| {
                println!("[ECOMM] Entregado {:>2} x {}", msg.quantity, msg.product_id);
                Ok(())
            },
        ))
    }
}

impl Handler<Print> for Shop {
    type Result = ();

    fn handle(&mut self, _msg: Print, _ctx: &mut Context<Self>) -> Self::Result {
        println!("{:?}", self);
    }
}
