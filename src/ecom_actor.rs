use crate::error::FileError;
use crate::messages::process_orders::ForwardOrder;
use actix::{Actor, AsyncContext, Context, StreamHandler};
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
    pub id: u8,
    pub stream: Option<Arc<Mutex<WriteHalf<TcpStream>>>>,
}

#[derive(Debug, Clone)]
pub struct EcomOrder {
    pub id: u8,
    pub product_id: String,
    pub quantity: u32,
    pub zone_id: u8,
    pub shops_requested: Vec<u8>,
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

    fn fetch_shop_streams() -> Result<Vec<(u8, TcpStream)>, FileError> {
        let mut streams: Vec<(u8, TcpStream)> = Vec::new();
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
            let location: u8 = shop_info[2].parse().unwrap();
            println!("{}", location);
            streams.push((location, TcpStream::from_std(stream).unwrap()));
        }
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

        let mut ecom = Self {
            name: ecom_info[0].to_string(),
            address: ecom_info[1].to_string(),
            pending_orders: Vec::new(),
            zones: Vec::new(),
        };

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

            ecom.pending_orders.push(ecom_order);
            line_number += 1;
        }

        Ok(ecom)
    }

    pub fn get_zone_to_send(&self, order: &EcomOrder) -> Option<Zone> {
        let mut zone_to_send: Option<Zone> = None;
        for zone in self.zones.iter() {
            match zone_to_send {
                None => {
                    if !order.shops_requested.contains(&zone.id) {
                        zone_to_send = Some(zone.clone());
                    }
                }
                Some(ref z) => {
                    if !order.shops_requested.contains(&zone.id)
                        && ((z.id - order.zone_id) as i32).abs()
                            > ((zone.id - order.zone_id) as i32).abs()
                    {
                        zone_to_send = Some(zone.clone());
                    }
                }
            }
        }
        zone_to_send
    }
    pub fn clear_requested_shops(&mut self, order_id: u8) {
        let order = self
            .pending_orders
            .iter_mut()
            .find(|order| order.id == order_id)
            .unwrap();

        order.shops_requested.clear();
    }
}

impl Actor for Ecom {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("INICIANDO ECOMMERCE");

        let streams = Self::fetch_shop_streams().unwrap();
        for (index, (id, stream)) in streams.into_iter().enumerate() {
            let (read, write_half) = split(stream);
            Ecom::add_stream(
                wrappers::LinesStream::new(tokio::io::BufReader::new(read).lines()),
                ctx,
            );

            self.zones.insert(
                index,
                Zone {
                    id: id.clone(),
                    stream: Some(Arc::new(Mutex::new(write_half))),
                },
            );
        }
    }
}

impl<'a> StreamHandler<Result<String, std::io::Error>> for Ecom {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(line) = read {
            let order_str = line.split(',').collect::<Vec<&str>>();
            let id = order_str[0].parse::<u8>().unwrap();
            let result = order_str[1].parse::<u8>().unwrap();
            let message = order_str[2].to_string();

            let order = self.pending_orders.iter().find(|o| o.id == id).unwrap();
            println!(
                "[ECOM] Pedido {} desde tienda [{:?}]: {:<2}x {}",
                message,
                order.shops_requested.last().unwrap(),
                order.quantity,
                order.product_id
            );

            match result {
                0 => ctx.address().try_send(ForwardOrder(order.clone())).unwrap(),
                1 => self.pending_orders.retain(|order| order.id != id),
                _ => (),
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        println!("[ECOM] Desconectado");
    }
}
