use std::time::Duration;

use actix::{dev::ContextFutureSpawner, fut::wrap_future, AsyncContext, Handler, Message};
use rand::{thread_rng, Rng};

use crate::constants::ECOM_PROCESING_MILLIS;

use super::{
    ecom_actor::{Ecom, EcomOrder},
    process_order::ProcessOrder,
};

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ProcessEcomOrders(pub Vec<EcomOrder>);

impl Handler<ProcessEcomOrders> for Ecom {
    type Result = ();

    fn handle(&mut self, mut msg: ProcessEcomOrders, ctx: &mut Self::Context) -> Self::Result {
        let next_order = match msg.0.first() {
            Some(order) => order.clone(),
            None => return,
        };
        let address = ctx.address().clone();

        wrap_future::<_, Self>(async move {
            tokio::time::sleep(Duration::from_millis(
                thread_rng().gen_range(ECOM_PROCESING_MILLIS),
            ))
            .await;
            address.do_send(ProcessOrder(next_order));

            msg.0.remove(0);

            if !msg.0.is_empty() {
                address.do_send(ProcessEcomOrders(msg.0));
            }
        })
        .spawn(ctx);
    }
}
