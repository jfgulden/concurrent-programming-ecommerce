use std::time::Duration;

use actix::clock::sleep;
use actix::{ActorFutureExt, Context, Handler, Message, ResponseActFuture, WrapFuture};
use rand::{thread_rng, Rng};

use crate::constants::DELIVER_MILLIS;
use crate::states::OnlinePurchaseState;

use super::{online_purchase::OnlinePurchase, shop_actor::Shop};

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct DeliverPurchase {
    pub purchase: OnlinePurchase,
}

impl Handler<DeliverPurchase> for Shop {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, mut msg: DeliverPurchase, _ctx: &mut Context<Self>) -> Self::Result {
        let delivery_time = Duration::from_millis(thread_rng().gen_range(DELIVER_MILLIS));

        Box::pin(
            sleep(delivery_time)
                .into_actor(self)
                .map(move |_msg, shop, ctx| {
                    msg.purchase.state.deliver_attempt();
                    msg.purchase.print_status();

                    let product = shop.stock.iter_mut().find(|p| p.id == msg.purchase.product);

                    if msg.purchase.state == OnlinePurchaseState::DELIVERED {
                        if let Some(product) = product {
                            product.reserved -= msg.purchase.quantity;
                        }

                        msg.purchase.send_msg(ctx);
                    } else {
                        if let Some(product) = product {
                            product.reserved -= msg.purchase.quantity;
                            product.stock += msg.purchase.quantity;
                        }
                    }
                }),
        )
    }
}
