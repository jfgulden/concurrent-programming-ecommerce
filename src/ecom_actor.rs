use crate::error::{FileError, PurchaseError};
use crate::messages::ecommerce_purchase::EcommercePurchase;
use crate::messages::print::Print;
use crate::messages::process_order::ProcessOrders;
use actix::{Actor, Context, Handler};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::thread;

#[derive(Debug)]
pub struct Product {
    pub id: String,
    pub stock: u32,
    pub reserved: u32,
}

#[derive(Debug)]
pub struct Ecom {
    pub name: String,
    pub address: String,
    pub orders: Vec<EcommercePurchase>,
    streams: HashMap<u8, TcpStream>, //Esto debería ser un HashMap<(lat, long) o zone_id, TcpStream>
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

    fn fetch_shop_streams() -> Result<HashMap<u8, TcpStream>, FileError> {
        let mut streams: HashMap<u8, TcpStream> = HashMap::new();
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

            let stream = TcpStream::connect(shop_info[1]).unwrap();
            let location: u8 = shop_info[2].parse().unwrap();
            println!("{}", location);
            streams.insert(location, stream);
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

        let streams = Ecom::fetch_shop_streams()?;

        let mut ecom = Self {
            name: ecom_info[0].to_string(),
            address: ecom_info[1].to_string(),
            orders: Vec::new(),
            streams,
        };

        // ignore dash line
        lines.next();
        for line in lines {
            let current_line = line.map_err(|_| FileError::WrongFormat)?;

            let product_data: Vec<&str> = current_line.split(',').collect();

            // ['KEY', 'VALUE', 'ZONE'].len() == 3
            if product_data.len() != 3 {
                return Err(FileError::WrongFormat);
            }
            let ecom_order = EcommercePurchase {
                product_id: product_data[0].to_string(),
                quantity: product_data[1]
                    .parse()
                    .map_err(|_| FileError::WrongFormat)?,
                zone_id: product_data[2]
                    .parse()
                    .map_err(|_| FileError::WrongFormat)?,
            };

            ecom.orders.push(ecom_order);
        }

        Ok(ecom)
    }
}

impl Actor for Ecom {
    type Context = Context<Self>;
}

impl Handler<ProcessOrders> for Ecom {
    type Result = Result<(), PurchaseError>;

    fn handle(&mut self, _msg: ProcessOrders, _ctx: &mut Self::Context) -> Self::Result {
        for (index, order) in self.orders.iter().enumerate() {
            //lógica para saber a donde mandar (que zona)
            //estaría bueno que el hashmap guarde lat y long y cada pedido tenga su lat y long.
            //de esa forma compararíamos las distancias y mandaríamos el pedido a la tienda más cercana
            let mut shop_stream = self.streams.get(&order.zone_id).unwrap();
            let message = if index == self.orders.len() - 1 {
                format!("{},{},{}", order.product_id, order.quantity, order.zone_id)
            } else {
                format!("{},{},{}/", order.product_id, order.quantity, order.zone_id)
            };
            let _bytes = shop_stream.write_all(message.as_bytes()).unwrap();
        }
        Ok(())
    }
}
impl Handler<EcommercePurchase> for Ecom {
    type Result = Result<(), PurchaseError>; //ResponseActFuture<Self,

    fn handle(&mut self, _msg: EcommercePurchase, _ctx: &mut Context<Self>) -> Self::Result {
        // thread::sleep(Duration::from_millis(thread_rng().gen_range(500..1500)));

        let _algo = self
            .orders
            .iter()
            .map(|order| {
                println!(
                    "[ECOMM]  Solicitamos stock:   {:>2} x {} a zona: {}",
                    order.quantity, order.product_id, order.zone_id
                );
                println!(
                    "[ECOMM]   Se entregaron:     {:>2} x {} desde zona: {}",
                    order.quantity, order.product_id, order.zone_id
                );
            })
            .collect::<Vec<_>>();
        Ok(())
    }
}

impl Handler<Print> for Ecom {
    type Result = ();

    fn handle(&mut self, _msg: Print, _ctx: &mut Context<Self>) -> Self::Result {
        println!("{:?}", self);
    }
}
