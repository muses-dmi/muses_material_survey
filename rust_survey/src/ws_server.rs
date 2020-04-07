//! Simple websocket server for Muses survey    
//! 
//! Copyright: Benedict R. Gaster
//! 
//! 

extern crate ws;

use std::thread;
use std::sync::mpsc::channel;
use std::io::stdin;

use ws::{Factory, Handler};

use std::sync::mpsc::{Sender, Receiver};
use serde_json::Value;

use crate::msg::*;

struct ServerHandler {
    ws: ws::Sender,
    inbound: Sender<Message>,
}

impl Handler for ServerHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> { 
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> { 
        match msg {
            ws::Message::Text(data) => {
                let v: Value = serde_json::from_str(&data).unwrap();
                info!("message received from client: {}", v.clone());
                // send message to main app
                self.inbound.send(v);
            },
            _ => {

            }
        }

        Ok(())
    }
}

struct ServerFactory {
    inbound: Sender<Message>,
}

impl Factory for ServerFactory {
    type Handler = ServerHandler;

    fn connection_made(&mut self, ws: ws::Sender) -> ServerHandler {
        ServerHandler {
            ws: ws,
            // default to server
            inbound: self.inbound.clone(),
        }
    }
}

pub struct WSServer {
    //socket : ws::WebSocket<ServerFactory>,
    handle: ws::Sender,
    listening_thread: thread::JoinHandle<()>,
}

impl WSServer {
    pub fn new(
        address: String, 
        inbound:Sender<Message>) -> Self {

        let socket : ws::WebSocket<ServerFactory> = 
            ws::Builder::new()
            .build(ServerFactory {
                //tx: tx,
                inbound: inbound,
        }).unwrap();

        // get a handle to sender so we can send messages outbound
        let handle = socket.broadcaster();

        // we need a thread to handle incomming events
        let ws_thread : thread::JoinHandle<()> = thread::spawn(move || {
            socket.listen(address).unwrap();
        });

        WSServer {
            handle: handle,
            listening_thread: ws_thread, 
        }
    }

    pub fn sender(&self) -> &ws::Sender {
        &self.handle
    } 

    /// send JSON data to socket
    pub fn send(&self, data: Message) {
        // in general this should not fail, but just in case...
        match serde_json::to_string(&data) {
            Ok(s) => {
                self.handle.send(s);
            },
            Err(_) => {

            }
        }
    }
}

// pub fn run(
//     address: String, 
//     inbound:Sender<Message>, 
//     receive_incoming_msgs: Receiver<Message>) {

//     //let (tx, rx) = channel();
//     let socket : ws::WebSocket<ServerFactory> = 
//         ws::Builder::new()
//         .build(ServerFactory {
//             //tx: tx,
//             inbound: inbound,
//     }).unwrap();

//     let handle = socket.broadcaster();

//     // we need a thread to handle incomming events
//     let ws_thread : thread::JoinHandle<()> = thread::spawn(move || {
//         socket.listen(address).unwrap();
//     });

//     println!("Press Enter to exit survey backend");
//     let mut input = String::new();
//     stdin().read_line(&mut input).unwrap(); 
// }
