use std::{thread, time::Duration};

use crate::{
    ecom_actor::{Ecom, EcomOrder},
    error::PurchaseError,
};
use actix::{dev::ContextFutureSpawner, fut::wrap_future, AsyncContext, Handler, Message};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Message)]
#[rtype(result = "Result<(), PurchaseError>")]
pub struct ForwardOrder(pub EcomOrder);

#[derive(Debug, Message)]
#[rtype(result = "Result<(), PurchaseError>")]
pub struct ProcessOrders();

// impl Handler<ProcessOrders> for Ecom {
//     type Result = Result<(), PurchaseError>;

//     fn handle(&mut self, _msg: ProcessOrders, ctx: &mut Self::Context) -> Self::Result {
//         for order in self.pending_orders.iter() {
//             ctx.address().try_send(ForwardOrder(order.clone())).unwrap();
//         }
//         Ok(())
//     }
// }

impl Handler<ForwardOrder> for Ecom {
    type Result = Result<(), PurchaseError>;

    fn handle(&mut self, msg: ForwardOrder, ctx: &mut Self::Context) -> Self::Result {
        let zone_to_send = self.find_delivery_zone(&msg.0);
        // if zone_to_send.is_none() {
        //     //Si no hay tiendas disponibles, esperamos 4 segundos y volvemos a intentar
        //     wrap_future::<_, Self>(async move {
        //         tokio::time::sleep(Duration::from_secs(4)).await;
        //     })
        //     .spawn(ctx);
        //     self.clear_requested_shops(msg.0.id);
        //     zone_to_send = self.find_delivery_zone(
        //         self.pending_orders
        //             .iter()
        //             .find(|order| order.id == msg.0.id)
        //             .unwrap(),
        //     );
        // }
        if zone_to_send.is_none() {
            return Ok(()); //Tiene que retornar error en realidad
                           //Print de no realizada
        }
        let zone = zone_to_send.unwrap();
        let message = format!(
            "{},{},{},{}\n",
            msg.0.id, msg.0.product_id, msg.0.quantity, msg.0.zone_id
        );

        let write_mutex = zone.stream.clone().unwrap();
        wrap_future::<_, Self>(async move {
            let mut write: tokio::sync::MutexGuard<
                '_,
                tokio::io::WriteHalf<tokio::net::TcpStream>,
            > = write_mutex.lock().await;
            let _bytes = write.write_all(message.as_bytes()).await.unwrap();
        })
        .wait(ctx);
        self.pending_orders
            .iter_mut()
            .find(|order| order.id == msg.0.id)
            .unwrap()
            .shops_requested
            .push(zone.id);

        println!(
            "[ECOM] Enviando pedido a tienda [{:?}]: {:<2}x {}",
            zone.id, msg.0.quantity, msg.0.product_id
        );

        Ok(())
    }
}
