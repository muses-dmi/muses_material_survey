extern crate rosc;

use std::net::{UdpSocket, SocketAddrV4};
use std::str::FromStr;
use rosc::{OscPacket, OscType};

use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::sync::mpsc::{Sender};
use serde_json::Value;

use crate::msg;

pub struct OSC {
    sender: Sender<msg::SenselMessage>,
    socket: UdpSocket,
}

unsafe impl Send for OSC {

}

impl OSC {

    const MATERIAL_PREFIX: &'static str = "/material";
    // const SYNTH_PREFIX: &'static str = "/synth/";
    // const EFFECT_PREFIX: &'static str = "/effect/";
    // const MASTER_PREFIX: &'static str = "/master/";

    pub fn new(sender:Sender<msg::SenselMessage>, socket: UdpSocket) -> Self {
        OSC {
            sender: sender,
            socket: socket,
        }
    }

    pub fn init(&mut self) {
        
    }

    fn match_int(v: OscType) -> i32 {
        match v {
            OscType::Int(i) => i,
            _ => {
                error!("expected OscType::Int");
                0
            }
        }
    }

    fn toFloat(osct: &OscType) -> f32 {
        match *osct {
            OscType::Float(f) => {
                f
            },
            _ => {
                0.0
            }
        }
    }

    fn toInt(osct: &OscType) -> u32 {
        match *osct {
            OscType::Int(i) => {
                i as u32
            },
            _ => {
                0
            }
        }
    }


    pub fn run(mut osc: OSC) {
        let sender = osc.sender.clone();
  
        let mut buf = [0u8; rosc::decoder::MTU];

        loop {
            match osc.socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    info!("Received osc packet with size {} from: {}", size, addr);
                    let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                    //handle_packet(packet);
                     match packet {
                        OscPacket::Message(msg) => { 
                            if msg.addr == OSC::MATERIAL_PREFIX {
                                    info!("received material message");
                                    // expecting two arguments (pressure and material index)
                                    match msg.args {
                                        Some(vargs) => {
                                            if vargs.len() == 5{
                                                osc.sender.send(
                                                    (msg::InputType::new(OSC::toInt(&vargs[0])), 
                                                     OSC::toFloat(&vargs[1]),
                                                     OSC::toFloat(&vargs[2]),
                                                     OSC::toFloat(&vargs[3]),
                                                     OSC::toInt(&vargs[4])));
                                            }
                                        },
                                        _ => {
                                            // ignore invalid message
                                        }

                                }
                            }
                        }
                        OscPacket::Bundle(bundle) => {
                            info!("OSC Bundle: {:?}", bundle);
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving from osc socket: {}", e);
                    break;
                }
            }
        }
    }
}

pub struct OSCBuilder {
    osc: OSC,
    initialized: bool,
}

impl OSCBuilder {
    //TODO: move these into config.json
    const IPADDRESS: &'static str = "127.0.0.1";
    const UDP_PORT: &'static str = "8338";

    pub fn new(sender: Sender<msg::SenselMessage>) -> Self {

        let address = format!("{}:{}", OSCBuilder::IPADDRESS, OSCBuilder::UDP_PORT);
        let addr = match SocketAddrV4::from_str(&address) {
            Ok(addr) => addr,
            Err(_) => panic!("failed to open socket"),
        };
    
        let sock = UdpSocket::bind(addr).unwrap();
        info!("Listening to {} for OSC input", addr);


        Self {
            osc: OSC::new(sender,sock),
            initialized: false,
        }
    }

    pub fn select_port(mut self) -> Self {
       
        self.osc.init();
        self.initialized = true;
        self
    }

    pub fn finish(self) -> OSC {
        if !self.initialized {
            panic!("failed to correctly initialized Application");
        }
        return self.osc;
    }
}