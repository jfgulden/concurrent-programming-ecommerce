use actix::fut::wrap_future;
use actix_rt::System;
use concurrentes::messages::ecom_purchase::EcomPurchase;
use concurrentes::{orders::Orders, shop_actor::Shop};
use rand::{thread_rng, Rng};
use std::io::{stdin, stdout, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{split, AsyncBufReadExt, BufReader, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::wrappers;
extern crate actix;
use actix::{Actor, ActorContext, Context, StreamHandler};
use futures::future::join_all;
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
            let purchase = EcomPurchase::parse(order, self.write.clone());
            self.shop_addr.try_send(purchase).unwrap();
            println!("Encolado");
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        println!("[ECOM] [{:?}] Desconectado", self.addr);
        ctx.stop();
    }
}

fn initiate_shop_side_server(shop: actix::Addr<Shop>, addr: &str) {
    let system = System::new();

    system.block_on(async {
        let listener: TcpListener = TcpListener::bind(addr).await.unwrap();
        while let Ok((stream, addr)) = listener.accept().await {
            println!("[ECOM] Se conectó el Ecommerce {:?}", addr);
            ShopSideServer::create(|ctx| {
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
    });

    system.run();
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

        let shop: Shop = match Shop::from_file(args[1].as_str()) {
            Ok(shop) => shop,
            Err(error) => {
                println!("ERROR shop: {:?}", error);
                return;
            }
        };

        println!("{:?}", shop);

        let orders = match Orders::from_file(args[2].as_str()) {
            Ok(orders) => orders,
            Err(error) => {
                println!("ERROR orders: {:?}", error);
                return;
            }
        };
        let address = shop.address.clone();
        let shop_actor = shop.start();
        let shop_clone: actix::Addr<Shop> = shop_actor.clone();

        let handle = thread::spawn(move || {
            initiate_shop_side_server(shop_clone, address.as_str());
        });

        // let orders_thread = tokio::spawn(async move {
        stdout().flush().unwrap();
        let mut input = String::new();
        println!("Presione enter para comenzar");
        stdin().read_line(&mut input).unwrap();

        for order in orders.list {
            let random = thread_rng().gen_range(100..=300);
            thread::sleep(Duration::from_millis(random));
            let _ = shop_actor.send(order).await.unwrap();
        }
        // });

        // join_all([server_thread, orders_thread]).await;

        handle.join().unwrap();

        println!("MAIN TERMINADO");
    });
    system.run().unwrap();
}
