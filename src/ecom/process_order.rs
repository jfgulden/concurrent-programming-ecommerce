use actix::{AsyncContext, Handler, Message};
use colored::Colorize;

use super::{
    ecom_actor::{Ecom, EcomOrder},
    foward_order::FowardOrder,
};

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ProcessOrder(pub EcomOrder);

impl Handler<ProcessOrder> for Ecom {
    type Result = ();

    fn handle(&mut self, msg: ProcessOrder, ctx: &mut Self::Context) -> Self::Result {
        let order = match self.pending_orders.get(&msg.0.id) {
            Some(order) => order,
            None => {
                self.pending_orders.insert(msg.0.id, msg.0.clone());
                self.pending_orders
                    .get(&msg.0.id)
                    .expect("ESTO NO DEBERIA OCURRIR")
            }
        };

        let shop_to_send = self.find_delivery_shop(order);

        let shop = match shop_to_send {
            Some(shop) => shop,
            None => {
                println!(
                    "{} Pedido {}: {:<2}x {} (No hay mas tiendas)",
                    "[ECOM]".purple(),
                    "CANCELADO".on_red(),
                    order.quantity,
                    order.product_id
                );
                // println!(
                //     "[ECOM] Pedido {:<2}x {} rechazado: No hay mas tiendas",
                //     order.quantity, order.product_id
                // );
                self.pending_orders.remove_entry(&msg.0.id);
                return;
            }
        };

        let order = match self.pending_orders.get_mut(&msg.0.id) {
            Some(order) => {
                order.shops_requested.push(shop.zone_id);
                order
            }
            None => return, // ESTO NO DEBERIA OCURRIR
        };

        ctx.address().do_send(FowardOrder {
            order: order.clone(),
            shop,
        });
    }
}
