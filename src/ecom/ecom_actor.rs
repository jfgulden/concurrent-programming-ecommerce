use crate::ecom::process_order::ProcessOrder;
use crate::error::FileError;
use crate::error::PurchaseError;
use crate::states::OnlinePurchaseState;
use actix::dev::ContextFutureSpawner;
use actix::fut::wrap_future;
use actix::{Actor, AsyncContext, Context, Message, StreamHandler};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::vec;

use super::conneted_shops::ConnectedShop;
#[derive(Debug, Message, Clone)]
#[rtype(result = "Result<(), PurchaseError>")]
pub struct EcomOrder {
    pub id: u32,
    pub product_id: String,
    pub quantity: u32,
    pub zone_id: i32,
    pub shops_requested: Vec<i32>,
}

#[derive(Debug)]
pub struct Ecom {
    pub name: String,
    pub address: String,
    pub pending_orders: HashMap<u32, EcomOrder>,
    pub shops: Vec<ConnectedShop>, //Esto debería ser un HashMap<(lat, long) o zone_id, TcpStream>
}

impl Ecom {
    pub fn from_file(path: &str) -> Result<Self, FileError> {
        let file = File::open(path).map_err(|_| FileError::NotFound)?;
        let ecom = Self::from_reader(file)?;
        println!("===");
        println!("Nombre:  {}", ecom.name);
        println!("Servidor:  {}", ecom.address);
        println!("===\n");
        Ok(ecom)
    }

    fn from_reader<T: Read>(content: T) -> Result<Self, FileError> {
        let reader = BufReader::new(content);

        let mut lines = reader.lines();

        let ecom_info_string = match lines.next() {
            Some(string) => string.map_err(|_| FileError::WrongFormat)?,
            None => return Err(FileError::WrongFormat),
        };
        let ecom_info: Vec<&str> = ecom_info_string.split(',').collect();
        if ecom_info.len() != 2 {
            return Err(FileError::WrongFormat);
        }

        let ecom = Self {
            name: ecom_info[0].to_string(),
            address: ecom_info[1].to_string(),
            pending_orders: HashMap::new(),
            shops: Vec::new(),
        };

        Ok(ecom)
    }

    pub fn orders_from_file(path: &str) -> Result<Vec<EcomOrder>, FileError> {
        let file = File::open(path).map_err(|_| FileError::NotFound)?;
        let reader = BufReader::new(file);

        let mut orders: Vec<EcomOrder> = Vec::new();
        let mut lines = reader.lines();

        // ignore info line
        lines.next();
        // ignore dash line
        lines.next();

        let mut line_number = 0;
        for line in lines {
            let current_line = line.map_err(|_| FileError::WrongFormat)?;

            let product_data: Vec<&str> = current_line.split(',').collect();

            // ['KEY', 'VALUE', 'ZONE'].len() == 3
            if product_data.len() != 3 {
                return Err(FileError::WrongFormat);
            }
            let ecom_order = EcomOrder {
                id: line_number,
                product_id: product_data[0].to_string(),
                quantity: product_data[1]
                    .parse()
                    .map_err(|_| FileError::WrongFormat)?,
                zone_id: product_data[2]
                    .parse()
                    .map_err(|_| FileError::WrongFormat)?,
                shops_requested: vec![],
            };

            orders.push(ecom_order);
            line_number += 1;
        }

        Ok(orders)
    }

    pub fn find_delivery_shop(&self, order: &EcomOrder) -> Option<ConnectedShop> {
        let mut shops = self.shops.clone();
        shops.sort_by(|a, b| {
            if (a.zone_id - order.zone_id).abs() > (b.zone_id - order.zone_id).abs() {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        });

        for shop in shops {
            if !order.shops_requested.contains(&shop.zone_id) {
                return Some(shop);
            }
        }
        None
    }
}

impl Actor for Ecom {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for Ecom {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(line) = read {
            let order_str = line.split(',').collect::<Vec<&str>>();
            let id = match order_str[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => return,
            };
            let state_number = match order_str[1].parse::<u8>() {
                Ok(state_number) => state_number,
                Err(_) => return,
            };
            let state = match OnlinePurchaseState::from_int(state_number) {
                Some(state) => state,
                None => return,
            };

            let order = match self.pending_orders.get(&id) {
                Some(order) => order,
                None => return, // El pedido ya fue entregado o cancelado, alargue el timeout
            };

            println!(
                "[TIENDA {:?}] Pedido {}: {:<2}x {}",
                order.shops_requested.last().unwrap_or(&0),
                state.to_string(),
                order.quantity,
                order.product_id
            );

            match state {
                OnlinePurchaseState::DELIVERED => {
                    self.pending_orders.remove_entry(&id);
                }
                _ => ctx.address().do_send(ProcessOrder(order.clone())),
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        println!("[ECOM] [{:?}] Desconectado", self.address);
    }
}
