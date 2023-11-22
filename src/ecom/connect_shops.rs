use std::sync::Arc;

use actix::{Context, Handler, Message, StreamHandler};
use tokio::{
    io::{split, AsyncBufReadExt},
    sync::Mutex,
};
use tokio_stream::wrappers;

use crate::error::StreamError;

use super::{conneted_shops::ConnectedShop, ecom_actor::Ecom};

#[derive(Debug, Message)]
#[rtype(result = "Result<(), StreamError>")]
pub struct ConnectShops;

impl Handler<ConnectShops> for Ecom {
    type Result = Result<(), StreamError>;

    fn handle(&mut self, mut _msg: ConnectShops, ctx: &mut Context<Self>) -> Self::Result {
        let streams = ConnectedShop::from_file("tiendas").map_err(|_| StreamError::CannotCall)?;

        for (name, zone_id, stream) in streams.into_iter() {
            let (read, write_half) = split(stream);
            Ecom::add_stream(
                wrappers::LinesStream::new(tokio::io::BufReader::new(read).lines()),
                ctx,
            );

            self.shops.push(ConnectedShop {
                name,
                zone_id,
                stream: Arc::new(Mutex::new(write_half)),
            });
        }

        Ok(())
    }
}
