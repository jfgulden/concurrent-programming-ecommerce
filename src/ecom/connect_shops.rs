use actix::{Context, Handler, Message};
use colored::Colorize;

use crate::error::StreamError;

use super::{connected_shops::ConnectedShop, ecom_actor::Ecom};

#[derive(Debug, Message)]
#[rtype(result = "Result<(), StreamError>")]
pub struct ConnectShops;

impl Handler<ConnectShops> for Ecom {
    type Result = Result<(), StreamError>;

    fn handle(&mut self, mut _msg: ConnectShops, ctx: &mut Context<Self>) -> Self::Result {
        let streams = ConnectedShop::from_file("tiendas").map_err(|_| StreamError::CannotCall)?;

        for (name, zone_id, stream) in streams.into_iter() {
            if self.connect_shop(ctx, name, zone_id, stream).is_err() {
                println!(
                    "{} Error al conectar la tienda {}",
                    "[ECOM]".purple(),
                    zone_id
                );
            } else {
                println!("{} Tienda {} conectada", "[ECOM]".purple(), zone_id);
            };
        }

        Ok(())
    }
}
