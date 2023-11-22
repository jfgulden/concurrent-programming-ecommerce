use std::io::ErrorKind;

use actix::{dev::ContextFutureSpawner, fut::wrap_future, Addr, Context, Handler, Message};
use colored::Colorize;
use tokio::io::AsyncWriteExt;

use crate::error::StreamError;

use super::{connected_shops::ConnectedShop, ecom_actor::Ecom};

/// Read Stop and Reconnect commands from stdin ("s<shop_id> and r<shop_id> respectively"), and send messages to the correspondent ecom actor in each case
pub fn connection_handling(ecom: Addr<Ecom>) {
    std::thread::spawn(move || {
        for line in std::io::stdin().lines().flatten() {
            let (command, shop_id) = match parse_command(line) {
                Ok((command, shop_id)) => (command, shop_id),
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };
            if command == "s" {
                ecom.do_send(Stop(shop_id));
            } else if command == "r" {
                ecom.do_send(Reconnect(shop_id));
            }
        }
    });
}

/// Parses a line from stdin to a command and a shop id
fn parse_command(line: String) -> Result<(String, i32), ErrorKind> {
    let line = line.as_str();
    let command = match line.get(0..=0) {
        Some(c) => c,
        None => return Err(ErrorKind::InvalidData),
    };

    let shop_id = match line.get(1..) {
        Some(id) => id.parse::<i32>(),
        None => return Err(ErrorKind::InvalidData),
    };

    match shop_id {
        Ok(id) => Ok((command.to_string(), id)),
        Err(_) => Err(ErrorKind::InvalidData),
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct Stop(i32);

impl Handler<Stop> for Ecom {
    type Result = ();

    ///Disconnects the shop with the given zone id from the ecom
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
                let _ = guard.shutdown().await;
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

/// Tries to reconnect the shop with the given zone id to the ecom
impl Handler<Reconnect> for Ecom {
    type Result = Result<(), StreamError>;

    fn handle(&mut self, msg: Reconnect, ctx: &mut Context<Self>) -> Self::Result {
        let zone_id = msg.0;

        let streams = ConnectedShop::from_file("tiendas").map_err(|_| StreamError::CannotCall)?;

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
