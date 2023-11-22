use std::{net::SocketAddr, sync::Arc};

use actix::{Actor, ActorContext, Context, Recipient, StreamHandler};
use futures::TryFutureExt;
use tokio::{
    io::{split, AsyncBufReadExt, BufReader, WriteHalf},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_stream::wrappers;

use crate::shop::online_purchase::OnlinePurchase;

pub struct ShopServerSide {
    pub write: Arc<Mutex<WriteHalf<TcpStream>>>,
    pub addr: SocketAddr,
    pub shop_recipient: Recipient<OnlinePurchase>, //Lo vamos a usar para mandar msg al actor
}

impl Actor for ShopServerSide {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for ShopServerSide {
    fn handle(&mut self, read: Result<String, std::io::Error>, _ctx: &mut Self::Context) {
        let ecom = self.addr.port().to_string();
        if let Ok(line) = read {
            let order = line.split(',').collect::<Vec<&str>>();
            let purchase = match OnlinePurchase::parse(order, ecom, self.write.clone()) {
                Ok(purchase) => purchase,
                Err(_) => return,
            };

            self.shop_recipient.do_send(purchase);
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        println!("[ECOM {:?}] Desconectado", self.addr.port());
        ctx.stop();
    }
}

pub async fn initiate_shop_server_side(
    shop_recipient: Recipient<OnlinePurchase>,
    address: String,
) -> Result<(), String> {
    let listener = TcpListener::bind(address.as_str())
        .map_err(|_| String::from("Error listening port"))
        .await?;

    while let Ok((stream, addr)) = listener.accept().await {
        println!("[ECOM] Se conectó el Ecommerce {:?}", addr);
        let shop_recipient = shop_recipient.clone();
        ShopServerSide::create(|ctx| {
            let (read, write_half) = split(stream);
            ShopServerSide::add_stream(
                wrappers::LinesStream::new(BufReader::new(read).lines()),
                ctx,
            );

            let write = Arc::new(Mutex::new(write_half));

            ShopServerSide {
                addr,
                write,
                shop_recipient,
            }
        });
    }

    Ok(())
}
