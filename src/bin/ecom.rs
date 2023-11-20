use actix::{Actor, System};
use concurrentes::ecom_actor::ConnectShops;
// use concurrentes::messages::process_orders::ProcessOrders;
use concurrentes::{ecom_actor::Ecom, messages::process_orders::ForwardOrder};
use std::io::stdin;
use std::thread;
use std::time::Duration;
use std::{env, path::Path};

const CANT_ARGS: usize = 2;

fn main() {
    let system = System::new();

    system.block_on(async {
        let args: Vec<String> = env::args().collect();
        if args.len() < CANT_ARGS {
            println!("ERROR: ecom file not provided");
            return;
        }
        let path_ecom = Path::new(&args[1]);
        if !path_ecom.exists() {
            println!("ERROR: path from shop information does not exist");
            return;
        }

        let ecom: Ecom = match Ecom::from_file(args[1].as_str()) {
            Ok(ecom) => ecom,
            Err(error) => {
                println!("ERROR ecom: {:?}", error);
                return;
            }
        };
        // let orders = Ecom::orders_from_file(args[1].as_str());
        println!("ECOM: {:?}", ecom);
        let ecom_actor = ecom.start();

        let mut input = String::new();
        println!("Presione enter para comenzar");
        stdin().read_line(&mut input).unwrap();

        ecom_actor.send(ConnectShops()).await.unwrap();

        let orders = Ecom::orders_from_file(args[1].as_str()).unwrap();

        for order in orders {
            thread::sleep(Duration::from_millis(100));
            ecom_actor.send(ForwardOrder(order)).await.unwrap();
        }
        // ecom_actor.try_send(ProcessOrders()).unwrap();

        println!("MAIN TERMINADO");
    });
    system.run().unwrap();
}
