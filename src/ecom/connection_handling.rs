use std::io::ErrorKind;

use actix::{dev::ContextFutureSpawner, fut::wrap_future, Addr, Context, Handler, Message};
use colored::Colorize;
use tokio::io::AsyncWriteExt;

use crate::error::StreamError;

use super::{connected_shops::ConnectedShop, ecom_actor::Ecom};

pub fn connection_handling(ecom: Addr<Ecom>) {
    std::thread::spawn(move || {
        for line in std::io::stdin().lines().flatten() {
            let (command, tienda_id) = match parse_command(line) {
                Ok((command, tienda_id)) => (command, tienda_id),
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };
            if command == "s" {
                ecom.do_send(Stop(tienda_id));
            } else if command == "r" {
                ecom.do_send(Reconnect(tienda_id));
            }
        }
    });
}

fn parse_command(line: String) -> Result<(String, i32), ErrorKind> {
    let line = line.as_str();
    let command = match line.get(0..=0) {
        Some(c) => c,
        None => return Err(ErrorKind::InvalidData),
    };

    let tienda_id = match line.get(1..) {
        Some(id) => id.parse::<i32>(),
        None => return Err(ErrorKind::InvalidData),
    };

    match tienda_id {
        Ok(id) => Ok((command.to_string(), id)),
        Err(_) => Err(ErrorKind::InvalidData),
    }
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
