use std::time::Duration;

use actix::{dev::ContextFutureSpawner, fut::wrap_future, AsyncContext, Handler, Message};
use rand::{thread_rng, Rng};
use tokio::time::sleep;

use crate::{constants::LOCAL_PROCESING_MILLIS, shop::shop_actor::Shop};

use super::local_purchase::LocalPurchase;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ProcessLocalOrders(pub Vec<LocalPurchase>);

impl Handler<ProcessLocalOrders> for Shop {
    type Result = ();

    /// Processes the given orders one by one recursively, sending them as messages to the shop
    /// to be processed
    fn handle(&mut self, mut msg: ProcessLocalOrders, ctx: &mut Self::Context) -> Self::Result {
        let next_order = match msg.0.first() {
            Some(order) => order.clone(),
            None => return,
        };
        let address = ctx.address().clone();

        wrap_future::<_, Self>(async move {
            sleep(Duration::from_millis(
                thread_rng().gen_range(LOCAL_PROCESING_MILLIS),
            ))
            .await;
            address.do_send(next_order);

            msg.0.remove(0);

            if !msg.0.is_empty() {
                address.do_send(ProcessLocalOrders(msg.0));
            }
        })
        .spawn(ctx);
    }
}
