use std::{thread, time::Duration};

use actix::{dev::ContextFutureSpawner, fut::wrap_future, Actor, Context, Handler, Message};
use actix_rt::System;
use tokio::time::sleep;

fn main() {
    let system = System::new();

    system.block_on(async {
        let test = Test { count: 10 }.start();

        test.send(Sleep(1)).await;
        // thread::sleep(Duration::from_secs(1));
        test.send(Sleep(2)).await;
        // thread::sleep(Duration::from_secs(1));
        test.send(Sleep(3)).await;
        thread::sleep(Duration::from_secs(1));
        test.send(Ping()).await.unwrap();
    });

    system.run().unwrap();
}

struct Test {
    pub count: u32,
}

impl Actor for Test {
    type Context = Context<Self>;
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct Sleep(u32);

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct Ping();

impl Handler<Sleep> for Test {
    type Result = ();

    fn handle(&mut self, msg: Sleep, ctx: &mut Self::Context) -> Self::Result {
        wrap_future::<_, Self>(async move {
            println!("[{}] Sleeping...", msg.0);
            sleep(std::time::Duration::from_secs(2)).await;
            println!("[{}]Awake!", msg.0);
        })
        .wait(ctx);
    }
}

impl Handler<Ping> for Test {
    type Result = ();

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        println!("Ping!");
    }
}
