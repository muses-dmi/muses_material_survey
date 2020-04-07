
#![feature(bind_by_move_pattern_guards)]

#[macro_use]
extern crate log;
extern crate simple_logger;

#[macro_use]
extern crate serde_derive;

extern crate serde;
#[macro_use]
extern crate serde_json;

extern crate rand;
use rand::Rng;

extern crate uuid;
use uuid::Uuid;

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::Duration;

use std::fs::{write, File};
use std::io::Read;

use std::fs::OpenOptions;
use std::io::prelude::*;

mod osc_device;
mod msg;
mod ws_server;
mod slide;
mod world;

use crate::msg::*;

//------------------------------------------------------------------------------

const muses_config: &'static str = "./assets/config.json";

//------------------------------------------------------------------------------

/// Muses configuration, read from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    id: u32,
    csv: String,
    likert_dir: String,
}

fn main() {
    // logging is only enabled for debug build
    //#[cfg(debug_assertions)]
    //simple_logger::init().unwrap();

    info!("Muses Survey Backend");

    // Read config file
    // TODO: this is not portable to non POSIX systems
    let config_path = format!("{}", muses_config);
    let mut config = String::new();
    let mut f = 
        File::open(config_path.clone())
        .expect("Unable open config file");
    f.read_to_string(&mut config).expect("Unable to read config file");
    std::mem::drop(f);

    // Deserialize config
    let mut config : Config  = serde_json::from_str(&config).expect("Invalid config file");

    let id = Uuid::new_v4();
    let likert_cvs = format!("{}{}.csv", config.likert_dir.clone(), id.to_hyphenated().to_string());
    let mut likert_file = OpenOptions::new()
        .write(true)
        //.append(true)
        .create(true)
        .open(likert_cvs)
        .unwrap();

    // add a new line to the CSV file, just to make sure we start on a new line
    //writeln!(file, "\n");

    let mut world = world::World::new(id, likert_file);

    // update id to next free and update config file
    config.id = config.id + 1;
    let j = serde_json::to_string(&config).unwrap();
    write(config_path, j).expect("Unable to write file");

    //println!("filename {}", config.csv);

    // setup OSC thread....

    // create commincation channel for server
    let (osc_s, osc_r)    = channel();

    let osc_thread = std::thread::Builder::new()
            .spawn(move || {
                info!("osc thread is running");
                let osc = osc_device::OSCBuilder::new(osc_s)
                    .select_port()
                    .finish();

                osc_device::OSC::run(osc);
            }).unwrap();

    // create the survey

    // Now we setup up webserver which handles the event loop

    // channel to receive messages from web-client
    let (inbound, receive_incoming_msgs)  : (Sender<Message>, Receiver<Message>) = channel();

    // webserver handles the connection between backend and web-client
    // note: the listener is assumed not be on the main thread and thus does not 
    // block us 
    let ws = ws_server::WSServer::new("127.0.0.1:8080".to_string(), inbound.clone());

    // wait for connection message from client, so we know protocol has begun
    match receive_incoming_msgs.recv() {
        Ok(data) => {
            if !is_connected(data) {
                println!("bad connecition message");
                return;
            }
        },
        _ => {},
    }

    let mut slides: Vec<Box<slide::Slide>> = vec![
        Box::new(slide::FrontMatter::new()),
        //Box::new(slide::Consent::new()),
        Box::new(slide::Tap::new(1, 20, 2.0, 30.0, 30.0, 30.0, 30.0)),
        //Box::new(slide::Press::new(1, 20, 2.0)),
    ];

    // let mut slides: Vec<Box<slide::Slide>> = vec![
    //     Box::new(slide::FrontMatter::new()),
    //     Box::new(slide::Consent::new()),
    //     Box::new(slide::Likert::new(1, "Tap".to_string())),
    //     Box::new(slide::Likert::new(1, "Press".to_string())),
    //     Box::new(slide::Likert::new(1, "Slider".to_string())),
    //     Box::new(slide::Likert::new(2, "Tap".to_string())),
    //     Box::new(slide::Likert::new(2, "Press".to_string())),
    //     Box::new(slide::Likert::new(2, "Slider".to_string())),
    //     Box::new(slide::Likert::new(3, "Tap".to_string())),
    //     Box::new(slide::Likert::new(3, "Press".to_string())),
    //     Box::new(slide::Likert::new(3, "Slider".to_string())),
    
    //     //Box::new(slide::Press::new(1, 10)),
    // ];


    // let mut slides: Vec<Box<slide::Slide>> = vec![
    //     Box::new(slide::FrontMatter::new()),
    //     Box::new(slide::Consent::new()),
    //     Box::new(slide::Likert::new(3, "TAP".to_string())),
    //     Box::new(slide::Response::new("most_accurate".to_string(), 1, most_accurate_num())),
    //     Box::new(slide::Response::new("most_comfortable".to_string(), 1, most_comfortable_num())),
    //     Box::new(slide::Response::new("most_responsive".to_string(), 1, most_responsive_num())),
    //     Box::new(slide::Response::new("order_favorite".to_string(), 4, order_favorite_num())),
    //     Box::new(slide::Press::new(1, 10)),
    // ];

    for slide in slides {
        slide.run(&mut world, &osc_r, &ws, &receive_incoming_msgs);
    }
}
