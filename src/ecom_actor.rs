use crate::error::ConnectionError;
use crate::error::FileError;
use crate::error::PurchaseError;
use crate::messages::ecom_purchase::EcomPurchaseState;
use crate::messages::process_orders::ForwardOrder;
use actix::Handler;
use actix::{Actor, AsyncContext, Context, Message, StreamHandler};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::sync::Arc;
use std::vec;
use tokio::io::{split, AsyncBufReadExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_stream::wrappers;

#[derive(Debug, Clone)]

pub struct Zone {
    pub id: i32,
    pub stream: Option<Arc<Mutex<WriteHalf<TcpStream>>>>,
}
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
    pub pending_orders: Vec<EcomOrder>,
    pub zones: Vec<Zone>, //Esto debería ser un HashMap<(lat, long) o zone_id, TcpStream>
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

    fn fetch_shop_streams() -> Result<Vec<(i32, TcpStream)>, FileError> {
        let mut streams: Vec<(i32, TcpStream)> = Vec::new();
        let location_files = fs::read_dir("tiendas").unwrap();

        for dir_entry in location_files {
            let file = File::open(dir_entry.unwrap().path());
            let reader = BufReader::new(file.unwrap());
            let mut lines = reader.lines();

            let shop_info_string = match lines.next() {
                Some(string) => string.map_err(|_| FileError::WrongFormat)?,
                None => return Err(FileError::WrongFormat),
            };
            let shop_info: Vec<&str> = shop_info_string.split(',').collect();
            println!("{:?}", shop_info);
            if shop_info.len() != 3 {
                return Err(FileError::WrongFormat);
            }

            let stream = std::net::TcpStream::connect(shop_info[1]).unwrap();
            let location: i32 = shop_info[2].parse().unwrap();
            println!("{}", location);
            streams.push((location, TcpStream::from_std(stream).unwrap()));
        }
        //This function panics if it is not called from within a runtime with IO enabled.
        // The runtime is usually set implicitly when this function is called from a future driven by a tokio runtime, otherwise runtime can be set explicitly with Runtime::enter function.

        Ok(streams)
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
            pending_orders: Vec::new(),
            zones: Vec::new(),
        };

        Ok(ecom)
    }

    pub fn find_delivery_zone(&self, order: &EcomOrder) -> Option<Zone> {
        let mut zones = self.zones.clone();
        zones.sort_by(|a, b| {
            if (a.id - order.zone_id).abs() > (b.id - order.zone_id).abs() {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        });

        for zone in zones {
            if !order.shops_requested.contains(&zone.id) {
                return Some(zone);
            }
        }
        None

        // let mut zone_to_send: Option<Zone> = None;
        // for zone in self.zones.iter() {
        //     match zone_to_send {
        //         None => {
        //             if !order.shops_requested.contains(&zone.id) {
        //                 zone_to_send = Some(zone.clone());
        //             }
        //         }
        //         Some(ref z) => {
        //             if !order.shops_requested.contains(&zone.id)
        //                 && ((z.id - order.zone_id) as i32).abs()
        //                     > ((zone.id - order.zone_id) as i32).abs()
        //             {
        //                 zone_to_send = Some(zone.clone());
        //             }
        //         }
        //     }
        // }
        // zone_to_send
    }
    pub fn clear_requested_shops(&mut self, order_id: u32) {
        let order = self
            .pending_orders
            .iter_mut()
            .find(|order| order.id == order_id)
            .unwrap();

        order.shops_requested.clear();
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), ConnectionError>")]
pub struct ConnectShops();

impl Handler<ConnectShops> for Ecom {
    type Result = Result<(), ConnectionError>;

    fn handle(&mut self, mut _msg: ConnectShops, ctx: &mut Context<Self>) -> Self::Result {
        println!("INICIANDO ECOMMERCE");

        let streams = Self::fetch_shop_streams().map_err(|_| ConnectionError::CannotCall)?;

        for (index, (id, stream)) in streams.into_iter().enumerate() {
            let (read, write_half) = split(stream);
            Ecom::add_stream(
                wrappers::LinesStream::new(tokio::io::BufReader::new(read).lines()),
                ctx,
            );

            self.zones.insert(
                index,
                Zone {
                    id,
                    stream: Some(Arc::new(Mutex::new(write_half))),
                },
            );
        }

        Ok(())
    }
}

impl Actor for Ecom {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {}
}

impl<'a> StreamHandler<Result<String, std::io::Error>> for Ecom {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(line) = read {
            let order_str = line.split(',').collect::<Vec<&str>>();
            let id = order_str[0].parse::<u32>().unwrap();
            let state = EcomPurchaseState::from_int(order_str[1].parse::<u8>().unwrap()).unwrap();

            let order = self.pending_orders.iter().find(|o| o.id == id).unwrap();
            println!(
                "[ECOM] Pedido {} desde tienda [{:?}]: {:<2}x {}",
                state.to_string(),
                order.shops_requested.last().unwrap(),
                order.quantity,
                order.product_id
            );

            match state {
                EcomPurchaseState::DELIVERED => self.pending_orders.retain(|order| order.id != id),
                EcomPurchaseState::LOST => {
                    ctx.address().try_send(ForwardOrder(order.clone())).unwrap()
                }
                EcomPurchaseState::REJECTED => {
                    ctx.address().try_send(ForwardOrder(order.clone())).unwrap()
                }
                _ => (),
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        println!("[ECOM] Desconectado");
    }
}
