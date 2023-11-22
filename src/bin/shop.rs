use actix_rt::System;
use concurrentes::error::FileError;
use concurrentes::shop::process_local_orders::ProcessLocalOrders;
use concurrentes::shop::shop_actor::Shop;
use concurrentes::shop::shop_server_side::initiate_shop_server_side;
use std::env;
use std::io::{stdin, stdout, Write};
extern crate actix;
use actix::Actor;
use std::path::Path;

const CANT_ARGS: usize = 2;

fn main() {
    let system = System::new();

    system.block_on(async {
        let (path_shop, path_orders) = match get_args() {
            Ok(args) => args,
            Err(_) => {
                System::current().stop();
                return;
            }
        };

        let shop = match Shop::from_file(path_shop.as_str()) {
            Ok(shop) => shop,
            Err(error) => {
                println!("ERROR creando Shop: {:?}", error);
                System::current().stop();
                return;
            }
        };

        let orders = match Shop::orders_from_file(path_orders.as_str()) {
            Ok(orders) => orders,
            Err(error) => {
                println!("ERROR creando Orders: {:?}", error);
                System::current().stop();
                return;
            }
        };

        enter_to_start();

        let address = shop.address.clone();
        let shop = shop.start();

        if let Err(err) = shop.send(ProcessLocalOrders(orders)).await {
            println!("ERROR: {:?}", err);
            System::current().stop();
            return;
        };
        if let Err(err) = initiate_shop_server_side(shop.recipient(), address).await {
            println!("ERROR: {:?}", err);
            System::current().stop()
        };
    });
    if system.run().is_err() {
        println!("ERROR: system error");
    }
}

/// Gets the path from the shop files from the command line arguments
fn get_args() -> Result<(String, String), FileError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < CANT_ARGS {
        println!("ERROR: shop files not provided");
        return Err(FileError::NotFound);
    }
    let path_shop = format!("tiendas/{}.txt", &args[1]);
    if !Path::new(&path_shop.as_str()).exists() {
        println!("ERROR: path from shop information does not exist");
        return Err(FileError::NotFound);
    }
    let path_orders = format!("pedidos/{}.txt", &args[1]);
    if !Path::new(&path_orders.as_str()).exists() {
        println!("ERROR: path from orders information does not exist");
        return Err(FileError::NotFound);
    }

    Ok((path_shop, path_orders))
}

/// Waits for the user to press enter to start the program
fn enter_to_start() {
    let _ = stdout().flush();
    let mut input = String::new();
    println!("Presione enter para comenzar");
    let _ = stdin().read_line(&mut input);
}
