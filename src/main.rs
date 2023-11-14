use concurrentes::messages::ecommerce_purchase::EcommercePurchase;
use concurrentes::messages::print::{self, Print};
use concurrentes::shop;
use concurrentes::{orders::Orders, shop::Shop};
use rand::{thread_rng, Rng};
extern crate actix;

use std::path::Path;
use std::time::Duration;
use std::{env, thread};

use actix::{Actor, System};

const CANT_ARGS: usize = 3;

// 1. Hacer tienda.txt para generar una Tienda con su info y stock
// agregar impl para crearlo

// 2. Hacer un structs para leer pedidos de un txt

// struct con vec<Pedidos> que tenga un impl crearlo

// 3. agregar prints piolas en los handlers y sleeps etc

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

        let shop_recipient = shop.clone();

        let mut a = thread_rng();

        thread::spawn(move || {
            let mut a_2 = thread_rng();
            loop {
                let random = a_2.gen_range(1000..=3000);
                thread::sleep(Duration::from_millis(random));
                shop_recipient
                    .try_send(EcommercePurchase {
                        product_id: "banana".to_string(),
                        quantity: 1,
                    })
                    .unwrap();
            }
        });

        for order in orders.list {
            let random = a.gen_range(1000..=3000);
            thread::sleep(Duration::from_millis(random));
            shop.send(order).await.unwrap();
        }

        shop.send(Print).await.unwrap();

        println!("MAIN TERMINADO");
    });
    system.run().unwrap();
}
