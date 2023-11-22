use actix_rt::System;
use concurrentes::error::FileError;
use concurrentes::shop::online_purchase::OnlinePurchase;
use concurrentes::shop::process_local_orders::ProcessLocalOrders;
use concurrentes::shop::shop_actor::Shop;
use futures::TryFutureExt;
use std::env;
use std::io::{stdin, stdout, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{split, AsyncBufReadExt, BufReader, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::wrappers;
extern crate actix;
use actix::{Actor, ActorContext, Context, StreamHandler};
use std::path::Path;

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
            let purchase = match OnlinePurchase::parse(order, self.write.clone()) {
                Ok(purchase) => purchase,
                Err(_) => return,
            };

            self.shop_addr.do_send(purchase);
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        println!("[ECOM] [{:?}] Desconectado", self.addr);
        ctx.stop();
    }
}

async fn initiate_shop_side_server(
    shop: &actix::Addr<Shop>,
    address: String,
) -> Result<(), String> {
    let listener: TcpListener = TcpListener::bind(address.as_str())
        .map_err(|_| String::from("Error listening port"))
        .await?;

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

    Ok(())
}
fn main() {
    let system = System::new();

    system.block_on(async {
        let args = match get_args() {
            Ok(args) => args,
            Err(_) => {
                System::current().stop();
                return;
            }
        };

        let shop = match Shop::from_file(args[1].as_str()) {
            Ok(shop) => shop,
            Err(error) => {
                println!("ERROR creando Shop: {:?}", error);
                System::current().stop();
                return;
            }
        };

        let orders = match Shop::orders_from_file(args[2].as_str()) {
            Ok(orders) => orders,
            Err(error) => {
                println!("ERROR creando Orders: {:?}", error);
                System::current().stop();
                return;
            }
        };

        enter_to_start();

        let address = shop.address.clone();
        let shop_actor = shop.start();

        if let Err(err) = shop_actor.send(ProcessLocalOrders(orders)).await {
            println!("ERROR: {:?}", err);
            System::current().stop();
            return;
        };
        if let Err(err) = initiate_shop_side_server(&shop_actor, address).await {
            println!("ERROR: {:?}", err);
            System::current().stop();
            return;
        };
        // server_fut.await;
    });
    if system.run().is_err() {
        println!("ERROR: system error");
    }
}

fn get_args() -> Result<Vec<String>, FileError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < CANT_ARGS {
        println!("ERROR: shop files not provided");
        return Err(FileError::NotFound);
    }
    let path_shop = Path::new(&args[1]);
    if !path_shop.exists() {
        println!("ERROR: path from shop information does not exist");
        return Err(FileError::NotFound);
    }
    let path_orders = Path::new(&args[2]);
    if !path_orders.exists() {
        println!("ERROR: path from orders information does not exist");
        return Err(FileError::NotFound);
    }

    return Ok(args);
}

fn enter_to_start() {
    let _ = stdout().flush();
    let mut input = String::new();
    println!("Presione enter para comenzar");
    let _ = stdin().read_line(&mut input);
}
