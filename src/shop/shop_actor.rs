use crate::error::FileError;
use crate::shop::local_purchase::LocalPurchase;
use crate::states::LocalPurchaseState;
use actix::{Actor, Context};

use std::fs::File;
use std::io::{BufRead, BufReader, Read};

#[derive(Debug)]
pub struct Product {
    pub id: String,
    pub stock: u32,
    pub reserved: u32,
}

pub struct Shop {
    pub name: String,
    pub address: String,
    pub location: u32,
    pub stock: Vec<Product>,
}

impl Shop {
    /// Reads the shop info from the file in the given path
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

    /// Reads the shop info from the given reader
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

    /// Reads the orders from the file in the given path
    pub fn orders_from_file(path: &str) -> Result<Vec<LocalPurchase>, FileError> {
        let file = File::open(path).map_err(|_| FileError::NotFound)?;
        let reader = BufReader::new(file);

        let mut orders = Vec::new();

        for line in reader.lines() {
            let current_line = line.map_err(|_| FileError::WrongFormat)?;
            let line_slices: Vec<&str> = current_line.split(',').collect();

            if line_slices.len() != 2 {
                return Err(FileError::WrongFormat);
            }

            let order = LocalPurchase {
                product: line_slices[0].to_string(),
                quantity: line_slices[1].parse().map_err(|_| FileError::WrongFormat)?,
                status: LocalPurchaseState::CREATED,
            };

            orders.push(order);
        }

        Ok(orders)
    }
}

impl Actor for Shop {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("INICIANDO TIENDA [{:?}]", self.location);
    }
}
