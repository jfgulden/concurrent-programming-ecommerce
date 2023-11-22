use std::time::Duration;

use crate::{constants::ECOM_MAX_WAITING_MILLIS, ecom::process_order::ProcessOrder};
use actix::{
    dev::ContextFutureSpawner, fut::wrap_future, ActorFutureExt, AsyncContext, Handler, Message,
    ResponseActFuture, WrapFuture,
};
use colored::Colorize;
use tokio::{io::AsyncWriteExt, time::sleep};

use super::{
    connected_shops::ConnectedShop,
    ecom_actor::{Ecom, EcomOrder},
};

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct FowardOrder {
    pub order: EcomOrder,
    pub shop: ConnectedShop,
}

impl Handler<FowardOrder> for Ecom {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: FowardOrder, ctx: &mut Self::Context) -> Self::Result {
        println!(
            "{} Enviando pedido a tienda en [{:?}]: {:<2}x {}",
            "[ECOM]".purple(),
            msg.shop.zone_id,
            msg.order.quantity,
            msg.order.product_id
        );

        let message = msg.order.as_string();

        wrap_future::<_, Self>(async move {
            let mut write = msg.shop.stream.lock().await;
            if write.write_all(message.as_bytes()).await.is_err() {
                println!(
                    "{} No se pudo enviar el pedido a la tienda en [{:?}]",
                    "[ECOM]".purple(),
                    msg.shop.zone_id
                );
            };
        })
        .wait(ctx);

        // timeout de perdida de pedido
        // solo se reenvia a otro si:
        //    - no se entrego y
        //    - no se mando el pedido a ninguna tienda mas
        // o sea, este pedido esta "perdido"
        return Box::pin(
            sleep(Duration::from_millis(ECOM_MAX_WAITING_MILLIS))
                .into_actor(self)
                .map(move |_, ecom, ctx| {
                    let order = match ecom.pending_orders.get(&msg.order.id) {
                        Some(order) => order,
                        None => return, // no es mas pendiente, ya se entrego o fue cancelada por no haber mas tiendas
                    };

                    if msg.shop.zone_id == *order.shops_requested.last().unwrap_or(&-1) {
                        println!("[ECOM] PERDIDO  {}x {}", order.quantity, order.product_id);
                        ctx.address().do_send(ProcessOrder(order.clone()));
                    } // caso contrario, sigue pendiente pero ya fue enviada a otra tienda
                }),
        );
    }
}
