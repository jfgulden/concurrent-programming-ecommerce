use actix::{Context, Handler, Message};

use crate::ecom_actor::Ecom;

#[derive(Debug, Message)]
#[rtype(result = "()")]

pub struct Print;

impl Handler<Print> for Ecom {
    type Result = ();

    fn handle(&mut self, _msg: Print, _ctx: &mut Context<Self>) -> Self::Result {
        println!("{:?}", self);
    }
}
