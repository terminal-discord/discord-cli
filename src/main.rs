use parking_lot::Mutex;
use serenity::{model::prelude::*, prelude::*};
use std::result::Result as StdResult;
use std::sync::mpsc::{Receiver, Sender};
use std::{env, sync::Arc, thread};

pub struct OneshotData {
    pub context: Context,
    pub ready: Ready,
}

pub struct OneshotHandler {
    tx: Arc<Mutex<Sender<OneshotData>>>,
}

impl OneshotHandler {
    pub fn new() -> (Receiver<OneshotData>, OneshotHandler) {
        let (tx, rx) = std::sync::mpsc::channel();

        let tx = Arc::new(Mutex::new(tx));
        (rx, OneshotHandler { tx })
    }
}

impl EventHandler for OneshotHandler {
    fn ready(&self, context: Context, ready: Ready) {
        {
            let mut ctx_lock = context.cache.write();
            for (&c_id, ch) in &ready.private_channels {
                if let Some(private) = ch.clone().private() {
                    ctx_lock.private_channels.insert(c_id, private);
                }
            }
        }
        let _ = self.tx.lock().send(OneshotData { context, ready });
    }
}

fn get_discord_ready(token: &str) -> StdResult<OneshotData, ()> {
    let (rx, handler) = OneshotHandler::new();

    let mut client = match Client::new(token, handler) {
        Ok(client) => client,
        Err(_err) => return Err(())?,
    };

    thread::spawn(move || {
        client.start_shards(1).unwrap();
    });

    rx.recv().map_err(|_| ())
}

fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let data = get_discord_ready(&token);
    println!("Logged in as {}", data.unwrap().ready.user.tag());
}
