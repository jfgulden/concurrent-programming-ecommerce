use std::time::Duration;

use actix::{
    dev::ContextFutureSpawner, fut::wrap_future, ActorFutureExt, Context, Handler, Message,
    ResponseActFuture, WrapFuture,
};
use rand::{thread_rng, Rng};
use tokio::{io::AsyncWriteExt, time::sleep};

use crate::shop_actor::Shop;

use super::ecommerce_purchase::EcommercePurchase;

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct DeliverPurchase {
    pub purchase: EcommercePurchase,
}

impl Handler<DeliverPurchase> for Shop {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: DeliverPurchase, _ctx: &mut Context<Self>) -> Self::Result {
        let delivery_time = Duration::from_millis(thread_rng().gen_range(500..1500));

        let delivery = sleep(delivery_time)
            .into_actor(self)
            .map(move |_result, _me, ctx| {
                let is_delivered = thread_rng().gen_bool(0.5);

                if is_delivered {
                    println!(
                        "[ECOMM]  Entregado {:>2} x {}",
                        msg.purchase.quantity, msg.purchase.product_id
                    );
                    wrap_future::<_, Self>(async move {
                        let mut write = msg.purchase.write.lock().await;
                        let msg = format!("{},1,ENTREGADO\n", msg.purchase.id);
                        write.write_all(msg.as_bytes()).await.unwrap();
                    })
                    .spawn(ctx);
                } else {
                    println!(
                        "[ECOMM]  Perdido   {:>2} x {}",
                        msg.purchase.quantity, msg.purchase.product_id
                    );
                    wrap_future::<_, Self>(async move {
                        let mut write = msg.purchase.write.lock().await;
                        let msg = format!("{},0,PERDIDO\n", msg.purchase.id);
                        write.write_all(msg.as_bytes()).await.unwrap();
                    })
                    .spawn(ctx);
                }
                return;
            });

        Box::pin(delivery)
    }
}
