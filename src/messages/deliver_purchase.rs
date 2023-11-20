use std::time::Duration;

use actix::{dev::ContextFutureSpawner, fut::wrap_future, Context, Handler, Message};
use rand::{thread_rng, Rng};
use tokio::io::AsyncWriteExt;

use crate::{error::PurchaseError, shop_actor::Shop};

use super::ecom_purchase::{EcomPurchase, EcomPurchaseState};

#[derive(Debug, Message, Clone)]
#[rtype(result = "Result<(), PurchaseError>")]
pub struct DeliverPurchase {
    pub purchase: EcomPurchase,
}

impl Handler<DeliverPurchase> for Shop {
    type Result = Result<(), PurchaseError>;

    fn handle(&mut self, mut msg: DeliverPurchase, ctx: &mut Context<Self>) -> Self::Result {
        let delivery_time = Duration::from_millis(thread_rng().gen_range(500..700));
        wrap_future::<_, Self>(async move {
            tokio::time::sleep(delivery_time).await;
            if msg.purchase.state != EcomPurchaseState::REJECTED {
                msg.purchase.state.set_delivery_status();
            }
            msg.purchase.print_status();

            let mut write = msg.purchase.write.lock().await;
            let msg = format!("{},{}\n", msg.purchase.id, msg.purchase.state.to_int());
            write.write_all(msg.as_bytes()).await.unwrap();
        })
        .spawn(ctx);

        Ok(())
    }
}
