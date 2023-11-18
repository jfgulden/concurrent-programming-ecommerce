use actix_rt::System;
use concurrentes::messages::ecommerce_purchase::EcommercePurchase;
use concurrentes::messages::print::Print;
use concurrentes::{orders::Orders, shop_actor::Shop};
use rand::{thread_rng, Rng};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{split, AsyncBufReadExt, BufReader, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::wrappers;
extern crate actix;
use actix::{Actor, ActorContext, Context, StreamHandler};
use std::path::Path;
use std::time::Duration;
use std::{env, thread};

const CANT_ARGS: usize = 3;

struct ShopSideServer {
    write: Arc<Mutex<WriteHalf<TcpStream>>>,
    addr: SocketAddr,
    shop_addr: actix::Addr<Shop>, //Lo vamos a usar para mandar msg al actor
}

impl Actor for ShopSideServer {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for ShopSideServer {
    fn handle(&mut self, read: Result<String, std::io::Error>, _ctx: &mut Self::Context) {
        if let Ok(line) = read {
            println!("[ECOM] Pedido recibido desde [{:?}]: {:?}", self.addr, line);

            let order = line.split(',').collect::<Vec<&str>>();
            let id = order[0].parse::<u8>().unwrap();
            let product_id = order[1].to_string();
            let quantity = order[2].parse::<u32>().unwrap();
            let zone_id = order[3].parse::<u8>().unwrap();
            let purchase = EcommercePurchase {
                id,
                product_id,
                quantity,
                zone_id,
                write: self.write.clone(),
            };

            let purchase_clone = purchase.clone();
            let shop_addr_clone = self.shop_addr.clone();

            shop_addr_clone.try_send(purchase_clone).unwrap();
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        println!("[ECOM] [{:?}] Desconectado", self.addr);
        ctx.stop();
    }
}

async fn initiate_shop_side_server(shop: actix::Addr<Shop>) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:2346").await.unwrap();
    while let Ok((stream, addr)) = listener.accept().await {
        println!("[ECOM] Se conectó el Ecommerce {:?}", addr);
        ShopSideServer::create(|ctx| {
            //Wrapper para que un actor pueda trabajar sobre un stream
            //Al actor le llega cada elemento del stream como si fuera un mensaje
            //LinesStream: Stream de líneas
            let (read, write_half) = split(stream);
            ShopSideServer::add_stream(
                wrappers::LinesStream::new(BufReader::new(read).lines()),
                ctx,
            );

            let write = Arc::new(Mutex::new(write_half));
            ShopSideServer {
                addr,
                write,
                shop_addr: shop.clone(),
            }
        });
    }
}
fn main() {
    let system = System::new();

    system.block_on(async {
        let args: Vec<String> = env::args().collect();
        if args.len() < CANT_ARGS {
            println!("ERROR: shop files not provided");
            return;
        }
        let path_shop = Path::new(&args[1]);
        if !path_shop.exists() {
            println!("ERROR: path from shop information does not exist");
            return;
        }
        let path_orders = Path::new(&args[2]);
        if !path_orders.exists() {
            println!("ERROR: path from orders information does not exist");
            return;
        }

        let shop: actix::Addr<Shop> = match Shop::from_file(args[1].as_str()) {
            Ok(shop) => shop,
            Err(error) => {
                println!("ERROR shop: {:?}", error);
                return;
            }
        }
        .start();

        let orders = match Orders::from_file(args[2].as_str()) {
            Ok(orders) => orders,
            Err(error) => {
                println!("ERROR orders: {:?}", error);
                return;
            }
        };
        let shop_clone = shop.clone();

        // let shop_recipient = shop.clone();
        let handle = thread::spawn(move || initiate_shop_side_server(shop_clone));

        let mut a = thread_rng();
        for order in orders.list {
            let random = a.gen_range(100..=300);
            thread::sleep(Duration::from_millis(random));
            let _ = shop.send(order).await.unwrap();
        }

        let _ = shop.send(Print).await;
        handle.join().unwrap().await;

        println!("MAIN TERMINADO");
    });
    system.run().unwrap();
}
