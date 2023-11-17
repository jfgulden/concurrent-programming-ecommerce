use std::{io::Write, thread, time::Duration};

use crate::{ecom_actor::Ecom, error::PurchaseError};
use actix::{dev::ContextFutureSpawner, fut::wrap_future, AsyncContext, Handler, Message};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Message)]
#[rtype(result = "Result<(), PurchaseError>")]
pub struct ProcessOrders();

impl Handler<ProcessOrders> for Ecom {
    type Result = Result<(), PurchaseError>;

    fn handle(&mut self, _msg: ProcessOrders, ctx: &mut Self::Context) -> Self::Result {
        for (_index, order) in self.orders.iter().enumerate() {
            //lógica para saber a donde mandar (que zona)
            //estaría bueno que el hashmap guarde lat y long y cada pedido tenga su lat y long.
            //de esa forma compararíamos las distancias y mandaríamos el pedido a la tienda más cercana
            let shop_write = self.streams.get_mut(&order.zone_id).unwrap();

            println!(
                "Se manda orden de {} x {} a tienda {}",
                order.quantity, order.product_id, order.zone_id
            );
            let message = format!(
                "{},{},{},{}\n",
                order.id, order.product_id, order.quantity, order.zone_id
            );

            let write_mutex = shop_write.clone();
            wrap_future::<_, Self>(async move {
                let mut write = write_mutex.lock().await;
                let _bytes = write.write_all(message.as_bytes()).await.unwrap();
            })
            .wait(ctx);

            // ESPERAR RESPUESTA

            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }
}
