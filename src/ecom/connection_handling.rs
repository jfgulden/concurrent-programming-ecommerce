use actix::{dev::ContextFutureSpawner, fut::wrap_future, Addr, Context, Handler, Message};
use colored::Colorize;
use tokio::io::AsyncWriteExt;

use crate::error::StreamError;

use super::{connected_shops::ConnectedShop, ecom_actor::Ecom};

pub fn connection_handling(ecom: Addr<Ecom>) {
    std::thread::spawn(move || {
        for line in std::io::stdin().lines() {
            if let Ok(command) = line {
                let (command, tienda_id) = parse_command(command);
                if command == "s" {
                    ecom.do_send(Stop(tienda_id));
                } else if command == "r" {
                    ecom.do_send(Reconnect(tienda_id));
                }
            }
        }
    });
}

fn parse_command(line: String) -> (String, i32) {
    let line = line.as_str();
    let command = line.get(0..=0).unwrap();

    let tienda_id: i32 = line.get(1..).unwrap().parse().unwrap();

    println!("Commando: {} para: {}", command, tienda_id);

    (command.to_string(), tienda_id)
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct Stop(i32);

impl Handler<Stop> for Ecom {
    type Result = ();

    fn handle(&mut self, msg: Stop, ctx: &mut Self::Context) -> Self::Result {
        let zone_id = msg.0;

        let mut shop_stream = None;
        for shop in &self.shops {
            if shop.zone_id == zone_id {
                shop_stream = Some(shop.stream.clone());
                break;
            }
        }
        self.shops.retain(|shop| shop.zone_id != zone_id);

        if let Some(shop_stream) = shop_stream {
            wrap_future::<_, Self>(async move {
                let mut guard = shop_stream.lock().await;
                guard.shutdown().await.unwrap();
            })
            .wait(ctx);
            println!("{} Desconectando tienda {}...", "[ECOM]".purple(), zone_id);
        } else {
            println!("No se encontro la tienda {}", zone_id);
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), StreamError>")]
struct Reconnect(i32);

impl Handler<Reconnect> for Ecom {
    type Result = Result<(), StreamError>;

    fn handle(&mut self, msg: Reconnect, ctx: &mut Context<Self>) -> Self::Result {
        let zone_id = msg.0;

        let streams = ConnectedShop::from_file().map_err(|_| StreamError::CannotCall)?;

        let mut new_shop = None;
        for shop in streams {
            if shop.1 == zone_id {
                new_shop = Some(shop);
                break;
            }
        }

        if let Some((name, zone_id, stream)) = new_shop {
            if self
                .connect_shop(ctx, name.clone(), zone_id, stream)
                .is_ok()
            {
                println!("{} Tienda {} reconectada", "[ECOM]".purple(), zone_id);
            } else {
                println!("Error al reconectar la tienda {}", zone_id);
            }
        } else {
            println!("No se encontro la tienda {}", zone_id);
        }

        Ok(())
    }
}
