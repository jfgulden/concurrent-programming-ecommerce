use actix::{Actor, Addr, System};
use concurrentes::ecom_actor::Ecom;
use concurrentes::messages::process_orders::ProcessOrders;
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

        let ecom: Addr<Ecom> = match Ecom::from_file(args[1].as_str()) {
            Ok(ecom) => ecom,
            Err(error) => {
                println!("ERROR ecom: {:?}", error);
                return;
            }
        }
        .start();

        // thread::sleep(Duration::from_millis(3000));
        ecom.try_send(ProcessOrders()).unwrap();

        println!("MAIN TERMINADO");
    });
    system.run().unwrap();
}
