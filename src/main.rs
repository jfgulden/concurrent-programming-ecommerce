use actix::clock::sleep;
use actix::Actor;
use concurrentes::{orders::Orders, shop::Shop};
use std::env;
use std::path::Path;

const CANT_ARGS: usize = 3;

// 1. Hacer tienda.txt para generar una Tienda con su info y stock
// agregar impl para crearlo

// 2. Hacer un structs para leer pedidos de un txt

// struct con vec<Pedidos> que tenga un impl crearlo

// 3. agregar prints piolas en los handlers y sleeps etc

#[actix_rt::main]
async fn main() {
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

    let shop = match Shop::from_file(args[1].as_str()) {
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

    for order in orders.list {
        let a = shop.send(order).await.unwrap();
    }
}
