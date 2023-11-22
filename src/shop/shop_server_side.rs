use std::{net::SocketAddr, sync::Arc};

use actix::{Actor, ActorContext, Context, StreamHandler};
use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};

use crate::shop::online_purchase::OnlinePurchase;

use super::shop_actor::Shop;

pub struct ShopServerSide {
    pub write: Arc<Mutex<WriteHalf<TcpStream>>>,
    pub addr: SocketAddr,
    pub shop_addr: actix::Addr<Shop>, //Lo vamos a usar para mandar msg al actor
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

            self.shop_addr.do_send(purchase);
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        println!("[ECOM {:?}] Desconectado", self.addr.port());
        ctx.stop();
    }
}
