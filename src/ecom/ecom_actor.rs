use crate::ecom::process_order::ProcessOrder;
use crate::error::FileError;
use crate::error::PurchaseError;
use crate::error::StreamError;
use crate::states::OnlinePurchaseState;
use actix::{Actor, AsyncContext, Context, Message, StreamHandler};
use colored::Colorize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::sync::Arc;
use std::vec;
use tokio::io::split;
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_stream::wrappers;

use super::connected_shops::ConnectedShop;
#[derive(Debug, Message, Clone)]
#[rtype(result = "Result<(), PurchaseError>")]
pub struct EcomOrder {
    pub id: u32,
    pub product_id: String,
    pub quantity: u32,
    pub zone_id: i32,
    pub shops_requested: Vec<i32>,
}

impl EcomOrder {
    pub fn as_string(&self) -> String {
        format!(
            "{},{},{},{}\n",
            self.id, self.product_id, self.quantity, self.zone_id
        )
    }
}

#[derive(Debug)]
pub struct Ecom {
    pub name: String,
    pub pending_orders: HashMap<u32, EcomOrder>,
    pub shops: Vec<ConnectedShop>,
}

impl Ecom {
    /// Reads the ecom info from the file in the given path
    pub fn from_file(path: &str) -> Result<Self, FileError> {
        let file = File::open(path).map_err(|_| FileError::NotFound)?;
        let ecom = Self::from_reader(file)?;
        println!("===");
        println!("Nombre:  {}", ecom.name);
        println!("===\n");
        Ok(ecom)
    }

    /// Reads the ecom info from the given reader
    fn from_reader<T: Read>(content: T) -> Result<Self, FileError> {
        let reader = BufReader::new(content);

        let mut lines = reader.lines();

        let ecom_info_string = match lines.next() {
            Some(string) => string.map_err(|_| FileError::WrongFormat)?,
            None => return Err(FileError::WrongFormat),
        };
        let ecom_info: Vec<&str> = ecom_info_string.split(',').collect();
        if ecom_info.len() != 1 {
            return Err(FileError::WrongFormat);
        }

        let ecom = Self {
            name: ecom_info[0].to_string(),
            pending_orders: HashMap::new(),
            shops: Vec::new(),
        };

        Ok(ecom)
    }

    /// Reads the orders from the file in the given path
    pub fn orders_from_file(path: &str) -> Result<Vec<EcomOrder>, FileError> {
        let file = File::open(path).map_err(|_| FileError::NotFound)?;
        let reader = BufReader::new(file);

        let mut orders: Vec<EcomOrder> = Vec::new();
        let mut lines = reader.lines();

        // ignore info line
        lines.next();
        // ignore dash line
        lines.next();

        for (line_number, line) in lines.enumerate() {
            let current_line = line.map_err(|_| FileError::WrongFormat)?;

            let product_data: Vec<&str> = current_line.split(',').collect();

            // ['KEY', 'VALUE', 'ZONE'].len() == 3
            if product_data.len() != 3 {
                return Err(FileError::WrongFormat);
            }
            let ecom_order = EcomOrder {
                id: line_number as u32,
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
        }

        Ok(orders)
    }

    /// Returns the next shop, if any, to deliver the order or None if there are no more shops available.
    pub fn find_delivery_shop(&self, order: &EcomOrder) -> Option<ConnectedShop> {
        let mut shops = self.shops.clone();
        shops.sort_by(|a, b| {
            if (a.zone_id - order.zone_id).abs() > (b.zone_id - order.zone_id).abs() {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        });

        shops
            .into_iter()
            .find(|shop| !order.shops_requested.contains(&shop.zone_id))
    }

    /// Tries to stablish a connection with the shop with the given zone id
    pub fn connect_shop(
        &mut self,
        ctx: &mut Context<Ecom>,
        name: String,
        zone_id: i32,
        address: String,
    ) -> Result<(), StreamError> {
        let stream_std = match std::net::TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(StreamError::CannotCall),
        };

        let stream = match TcpStream::from_std(stream_std) {
            Ok(stream) => stream,
            Err(_) => return Err(StreamError::CannotCall),
        };

        let (read, write_half) = split(stream);
        Ecom::add_stream(
            wrappers::LinesStream::new(tokio::io::BufReader::new(read).lines()),
            ctx,
        );

        self.shops.push(ConnectedShop {
            name,
            zone_id,
            stream: Arc::new(Mutex::new(write_half)),
        });

        Ok(())
    }
}

impl Actor for Ecom {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for Ecom {
    /// Handles the messages received from the shops for each order
    /// If the order is DELIVERED, it removes it from the pending orders
    /// If the order is not DELIVERED, it sends a ProcessOrder message to the ecom
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
                "{} Pedido {}: {:<2}x {}",
                format!("[TIENDA {}]", order.shops_requested.last().unwrap_or(&-1)).blue(),
                state.string_to_print(),
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

    fn finished(&mut self, _ctx: &mut Self::Context) {}
}

#[cfg(test)]
mod tests {

    use std::{sync::Arc, thread, time::Duration};
    use tokio::{io::split, net::TcpStream, sync::Mutex};

    use crate::ecom::ecom_actor::EcomOrder;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[actix_rt::test]
    async fn test_find_delivery_zone() {
        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("localhost:12355").unwrap();
            listener.accept().unwrap();
        });
        thread::sleep(Duration::from_millis(100));
        let stream = std::net::TcpStream::connect("localhost:12355").unwrap();
        let tokio_stream = TcpStream::from_std(stream).unwrap();
        let (_read, write) = split(tokio_stream);
        let write = Arc::new(Mutex::new(write));
        let conneted_shops = vec![
            ConnectedShop {
                name: "retiro".to_string(),
                zone_id: 1,
                stream: write.clone(),
            },
            ConnectedShop {
                name: "palermo".to_string(),
                zone_id: 5,
                stream: write.clone(),
            },
            ConnectedShop {
                name: "recoleta".to_string(),
                zone_id: 11,
                stream: write.clone(),
            },
            ConnectedShop {
                name: "belgrano".to_string(),
                zone_id: 20,
                stream: write,
            },
        ];
        let ecom = Ecom {
            name: String::from("ecom"),
            pending_orders: HashMap::new(),
            shops: conneted_shops,
        };

        let order1 = EcomOrder {
            id: 1,
            product_id: String::from("1"),
            quantity: 1,
            zone_id: 1,
            shops_requested: vec![],
        };
        let order4 = EcomOrder {
            id: 1,
            product_id: String::from("1"),
            quantity: 1,
            zone_id: 4,
            shops_requested: vec![],
        };
        let order7 = EcomOrder {
            id: 1,
            product_id: String::from("1"),
            quantity: 1,
            zone_id: 7,
            shops_requested: vec![],
        };
        let order15 = EcomOrder {
            id: 1,
            product_id: String::from("1"),
            quantity: 1,
            zone_id: 15,
            shops_requested: vec![],
        };
        let shop1 = ecom.find_delivery_shop(&order1).unwrap();
        let shop4 = ecom.find_delivery_shop(&order4).unwrap();
        let shop7 = ecom.find_delivery_shop(&order7).unwrap();
        let shop15 = ecom.find_delivery_shop(&order15).unwrap();

        assert_eq!(shop1.zone_id, 1);
        assert_eq!(shop4.zone_id, 5);
        assert_eq!(shop7.zone_id, 5);
        assert_eq!(shop15.zone_id, 11);
    }
}
