use actix::{Actor, System};
use concurrentes::ecom::connect_shops::ConnectShops;
use concurrentes::ecom::connection_handling::connection_handling;
use concurrentes::ecom::ecom_actor::Ecom;
use concurrentes::ecom::process_ecom_orders::ProcessEcomOrders;
use concurrentes::error::FileError;
// use concurrentes::messages::process_orders::ProcessOrders;
use std::io::{stdin, stdout, Write};
use std::{env, path::Path};

const CANT_ARGS: usize = 2;

fn main() {
    let system = System::new();

    system.block_on(async {
        let path = match get_args() {
            Ok(args) => args,
            Err(_) => {
                System::current().stop();
                return;
            }
        };
        let ecom: Ecom = match Ecom::from_file(path.as_str()) {
            Ok(ecom) => ecom,
            Err(error) => {
                println!("ERROR ecom: {:?}", error);
                System::current().stop();
                return;
            }
        };
        let ecom = ecom.start();

        start_on_enter();

        if let Err(error) = ecom.send(ConnectShops).await {
            println!("ERROR conectando shops: {:?}", error);
            System::current().stop();
            return;
        };

        let orders = match Ecom::orders_from_file(path.as_str()) {
            Ok(orders) => orders,
            Err(error) => {
                println!("ERROR obteniendo orders: {:?}", error);
                System::current().stop();
                return;
            }
        };

        connection_handling(ecom.clone());

        if let Err(error) = ecom.send(ProcessEcomOrders(orders)).await {
            println!("ERROR procesando ordenes: {:?}", error);
            System::current().stop()
        };
    });
    if system.run().is_err() {
        println!("ERROR: system error");
    }
}

/// Gets the path from the ecom file from the command line arguments
fn get_args() -> Result<String, FileError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < CANT_ARGS {
        println!("ERROR: ecom file not provided");
        return Err(FileError::NotFound);
    }

    let path = format!("pedidos/{}.txt", &args[1]);
    let path_ecom = Path::new(path.as_str());
    if !path_ecom.exists() {
        println!("ERROR: path from shop information does not exist");
        return Err(FileError::NotFound);
    }
    Ok(path)
}

/// Waits for the user to press enter to start the program
fn start_on_enter() {
    println!("Presione enter para comenzar");
    let _ = stdout().flush();
    let mut input = String::new();
    let _ = stdin().read_line(&mut input);
}
