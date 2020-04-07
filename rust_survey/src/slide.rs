//! Slide    
//! 
//! Copyright: Benedict R. Gaster
//! 
//! 

use rand::Rng;

use std::time::{Duration, Instant};
use std::sync::mpsc::{channel, Sender, Receiver};

use crate::world::*;

use crate::msg;

use crate::ws_server;
use crate::world;

use std::io::prelude::*;

//-----------------------------------------------------------------------------
// Utils
//-----------------------------------------------------------------------------

fn round (f: f32) -> f32 {
    f.floor() + 0.5
}

fn range(output_start: f32, output_end: f32, input_start: f32, input_end: f32, input: f32) -> f32 {
    let slope = 1.0 * (output_end - output_start) / (input_end - input_start);
    output_start + round(slope * (input - input_start))
} 

//-----------------------------------------------------------------------------
// SLIDES
//-----------------------------------------------------------------------------

pub trait Slide {
    fn run(&self,
        world: &mut world::World,
        inbound_osc: &Receiver<msg::SenselMessage>,
        outbound_msg: &ws_server::WSServer, 
        inbound_msg:  &Receiver<msg::Message>);
}

//-----------------------------------------------------------------------------
// Frontmatter
//-----------------------------------------------------------------------------

/// Front page of survey presentation
pub struct FrontMatter {
}

impl FrontMatter {
    pub fn new() -> Self {
        FrontMatter {

        }
    }
}

impl Slide for FrontMatter {
    fn run(&self, 
        world: &mut world::World,
        inbound_osc: &Receiver<msg::SenselMessage>,
        outbound_msg: &ws_server::WSServer, 
        inbound_msg:  &Receiver<msg::Message>) {

        // jump to frontmatter page
        outbound_msg.send(msg::gotoFrontMatter());

        // wait for user to press begin
        loop {
            match inbound_msg.recv() {
                Ok(data) => {
                    if msg::is_begin(data) {
                        return;
                    }
                },
                _ => {},
            }
        }
    }
}

/// Front page of survey presentation
pub struct Consent {
}

impl Consent {
    pub fn new() -> Self {
        Consent {
        }
    }
}

impl Slide for Consent {
    fn run(&self, 
        world: &mut world::World,
        inbound_osc: &Receiver<msg::SenselMessage>,
        outbound_msg: &ws_server::WSServer, 
        inbound_msg:  &Receiver<msg::Message>) {

        // jump to consent page
        outbound_msg.send(msg::consentID(world.create_id()));
        outbound_msg.send(msg::gotoConsent());

        // wait for user to press begin
        loop {
            match inbound_msg.recv() {
                Ok(data) => {
                    if msg::is_consent(data) {
                        return;
                    }
                },
                _ => {},
            }
        }
    }
}


pub struct Likert {
    material: u32,
    gesture: String,
}

impl Likert {
    pub fn new(material: u32, gesture: String) -> Self {
        Likert {
            material: material,
            gesture: gesture,
        }
    }
}

impl Slide for Likert {
    fn run(&self, 
        world: &mut world::World,
        inbound_osc: &Receiver<msg::SenselMessage>,
        outbound_msg: &ws_server::WSServer, 
        inbound_msg:  &Receiver<msg::Message>) {

        // set material and gesture
        outbound_msg.send(msg::materialIndex(self.material, msg::likert_num()));
        outbound_msg.send(msg::gestureType(self.gesture.clone()));

        // jump to likert page
        outbound_msg.send(msg::gotoLikert());

        // now wait for Likert response (3 likert messages expected)
        let mut msgs_received = 3;
        loop {
            match inbound_msg.recv() {
                Ok(data) => {
                    match msg::likert(data) {
                        Ok(l) => {
                            world.writeLikert(&self.gesture, &self.material.to_string(), l);
                            msgs_received = msgs_received - 1;
                        },
                        _ => {}
                    }
                },
                _ => {},
            }

            if msgs_received == 0 {
                return;
            }
        }
    }
}

/// Press page of survey presentation
pub struct Press {
    material: u32,
    duration: u64,
    tolerance: f32,
    // max_circle_adius: u32,
    // max_ring_adius: u32,
}

impl Press {
    const OUTPUT_START: f32 = 20.0;
    const OUTPUT_RING_MIN: f32 = 30.0;
    const OUTPUT_END: f32   = 100.0;
    const INPUT_START: f32  = 20.0;
    const INPUT_END: f32    = 1500.0;

    pub fn new(material: u32, duration: u64, tolerance: f32) -> Self {
        Press {
            material: material,
            duration: duration,
            tolerance: tolerance,
        }
    }
}


impl Slide for Press { 
    fn run(&self, 
        world: &mut world::World,
        inbound_osc: &Receiver<msg::SenselMessage>,
        outbound_msg: &ws_server::WSServer, 
        inbound_msg:  &Receiver<msg::Message>) {

        // jump to frontmatter page
        outbound_msg.send(msg::materialIndex(self.material, msg::press_num()));
        outbound_msg.send(msg::gotoPress());

        let mut rng = rand::thread_rng();

        let circle_radius     = Press::OUTPUT_START;
        let mut ring_radius   = rng.gen_range(Press::OUTPUT_RING_MIN, Press::OUTPUT_END);

        outbound_msg.send(msg::press(circle_radius, ring_radius));

        // track if touch is causing circle radius ~ ring radius, within a given tolerance
        let mut tolerance_timer = Instant::now();
        let mut within_tolerance = false;

        let overall_timer  = Instant::now();
        
        let mut data: Vec<world::Contacts> = vec![vec![]]; 
        let mut circle_ring_radius: Vec<(f32, f32)> = vec![(circle_radius, ring_radius)];
        let mut num_presses = 0;

        // pressure input, until time is done
        while overall_timer.elapsed().as_secs() < self.duration {
            
            match inbound_osc.try_recv() {
                Ok((input_type, pressure, x, y, material)) => {
                    if material == self.material {
                        // map pressure into range and then send radius to frontend
                        let circle_radius = range(
                            Press::OUTPUT_START, 
                            Press::OUTPUT_END, 
                            Press::INPUT_START, 
                            Press::INPUT_END, 
                            pressure);
                            
                        data[num_presses].push((overall_timer.elapsed().as_millis(), pressure, x, y));

                        // is circle radius ~ ring radius
                        if (ring_radius - circle_radius).abs() <= self.tolerance {
                            if within_tolerance {
                                if tolerance_timer.elapsed().as_secs() >= 1 {
                                    ring_radius = rng.gen_range(Press::OUTPUT_RING_MIN, Press::OUTPUT_END);
                                    within_tolerance = false;

                                    // update press storage
                                    num_presses = num_presses + 1;
                                    data.push(vec![]);
                                    circle_ring_radius.push((circle_radius, ring_radius));
                                }
                            }
                            else {
                                within_tolerance = true;
                                tolerance_timer = Instant::now();
                            }
                        }
                        else {
                            // need to make sure we reset seen tolerance if we fall out
                            within_tolerance = false;
                        }

                        outbound_msg.send(msg::press(circle_radius, ring_radius));
                    }
                },
                _ => {},
            }
        }
        world.writeGesture("press".to_string(), self.material, circle_ring_radius, data);
    }
}

//-----------------------------------------------------------------------------
// Slider gesture
//
// The implementation is very similar to press, but we are now dealing with 
// horx movements, rather than pressure, and the animation is different.
//-----------------------------------------------------------------------------

/// Slider page of survey presentation
pub struct Slider {
    /// material index
    material: u32,
    /// duration to test
    duration: u64,
    tolerance: f32,
    /// top left x of the pad
    top_left_x: f32,
    /// top left y of the pad
    top_left_y: f32,
    /// width of pad
    width: f32,
    /// height of pad
    height: f32,
}

impl Slider {
    const MIN_X: f32       = 50.0;
    const MAX_X: f32       = 650.0;
    const USER_X_SIZE: f32 = 20.0;
    const BOX_X_SIZE:  f32 = 150.0;
    const MIN_BOX_X_SIZE: f32 = Slider::USER_X_SIZE * 3.0;  
    const STEP_SIZE: f32   = 2.0;

    pub fn new(
        material: u32, 
        duration: u64, 
        tolerance: f32,
        top_left_x: f32,
        top_left_y: f32,
        width: f32,
        height: f32,) -> Self {
        Slider {
            material: material,
            duration: duration,
            tolerance: tolerance,
            top_left_x: top_left_x,
            top_left_y: top_left_y,
            width: width,
            height: height,
        }
    }
}

impl Slide for Slider { 
    fn run(&self, 
        world: &mut world::World,
        inbound_osc: &Receiver<msg::SenselMessage>,
        outbound_msg: &ws_server::WSServer, 
        inbound_msg:  &Receiver<msg::Message>) {

        // jump to frontmatter page
        outbound_msg.send(msg::materialIndex(self.material, msg::slider_num()));
        outbound_msg.send(msg::gotoSlider());

        // random number generator for direction and placement of box
        let mut rng = rand::thread_rng();

        let mut direction: f32 = rng.gen();
        direction = if direction > 90.0 { Slider::STEP_SIZE } else { -1.0 * Slider::STEP_SIZE };

        let mut user_x = rng.gen_range(Slider::MIN_X, Slider::MAX_X-Slider::USER_X_SIZE);
        
        let mut box_x = (rng.gen_range(Slider::MIN_X, Slider::MAX_X-Slider::BOX_X_SIZE) as u32) as f32;
        box_x = Slider::MIN_X;
        let mut box_size = Slider::BOX_X_SIZE;

        // place box and user box in initial positions 
        outbound_msg.send(msg::slider(user_x, box_x, box_size));

        // track if touch is causing circle radius ~ ring radius, within a given tolerance
        let mut tolerance_timer = Instant::now();
        let mut within_tolerance = false;

        // timer for time stamps outputs
        let overall_timer  = Instant::now();
        
        let mut data: Vec<world::Contacts> = vec![vec![]]; 
        let mut box_details: Vec<(f32, f32)> = vec![(box_x, box_size)];
        let mut num_boxes = 0;

        // timer to control animation FPS
        let mut animation_timer = Instant::now();

        // time animation and responses
        while overall_timer.elapsed().as_secs() < self.duration {   
            match inbound_osc.try_recv() {
                Ok((input_type, pressure, x, y, material)) => {
                    if material == self.material {
                        // update user box postion with respect to sensel input
                        user_x = range(
                            Slider::MIN_X,
                            Slider::MAX_X - Slider::USER_X_SIZE,
                            self.top_left_x,
                            self.top_left_x + self.width, 
                            x);

                        data[num_boxes].push((overall_timer.elapsed().as_millis(), pressure, x, y));

                        // is the user box inside the box?
                        if user_x >= box_x  && user_x+Slider::USER_X_SIZE <= box_x + box_size {
                            box_size = f32::max(box_size - 0.6, Slider::MIN_BOX_X_SIZE); 
                            box_x = box_x + if direction < 0.0 { -0.5 } else { 0.5 };
                            //box_x    = rng.gen_range(Slider::MIN_X, Slider::MAX_X - box_size);

                            //rng.gen_range(Slider::USER_X_SIZE, Slider::BOX_X_SIZE);
                            //box_x    = rng.gen_range(Slider::MIN_X, Slider::MAX_X - Slider::BOX_X_SIZE);
                            // update box storage
                            num_boxes = num_boxes + 1;
                            data.push(vec![]);
                            box_details.push((box_x, box_size));
                        }
                    }
                },
                _ => {}
            }

            // update box and its direction @60hz
            if animation_timer.elapsed().as_millis() > 16 {
                box_x = box_x + direction;

                // clamp within animation range and change direction of box if at boundary
                if box_x + box_size >= Slider::MAX_X  {
                    direction = -1.0 * Slider::STEP_SIZE;
                    box_x = Slider::MAX_X - box_size;
                }
                else if box_x <= Slider::MIN_X {
                    direction = Slider::STEP_SIZE;
                    box_x = Slider::MIN_X;
                }

                // reset timer
                animation_timer = Instant::now();

                // update view
                outbound_msg.send(msg::slider(user_x, box_x, box_size));
            }            
        }

        world.writeGesture("slider".to_string(), self.material, box_details, data);
    }
}

//-----------------------------------------------------------------------------
// Tap gesture
//
// The implementation is very similar to press, but we are now dealing with 
// horx movements, rather than pressure, and the animation is different.
//-----------------------------------------------------------------------------

/// Slider page of survey presentation
pub struct Tap {
    /// material index
    material: u32,
    /// duration to test
    duration: u64,
    tolerance: f32,
    /// top left x of the pad
    top_left_x: f32,
    /// top left y of the pad
    top_left_y: f32,
    /// width of pad
    width: f32,
    /// height of pad
    height: f32,
}

impl Tap {
    const MIN_X: f32       = 50.0;
    const MAX_X: f32       = 650.0;
    const USER_X_SIZE: f32 = 20.0;
    const BOX_X_SIZE:  f32 = 150.0;
    const MIN_BOX_X_SIZE: f32 = Tap::USER_X_SIZE * 3.0;  
    const STEP_SIZE: f32   = 2.0;

    pub fn new(
        material: u32, 
        duration: u64, 
        tolerance: f32,
        top_left_x: f32,
        top_left_y: f32,
        width: f32,
        height: f32,) -> Self {
        Tap {
            material: material,
            duration: duration,
            tolerance: tolerance,
            top_left_x: top_left_x,
            top_left_y: top_left_y,
            width: width,
            height: height,
        }
    }
}

impl Slide for Tap { 
    fn run(&self, 
        world: &mut world::World,
        inbound_osc: &Receiver<msg::SenselMessage>,
        outbound_msg: &ws_server::WSServer, 
        inbound_msg:  &Receiver<msg::Message>) {

        // jump to frontmatter page
        outbound_msg.send(msg::materialIndex(self.material, msg::tap_num()));
        outbound_msg.send(msg::gotoTap());

        // random number generator for direction and placement of box
        let mut rng = rand::thread_rng();

        let mut direction: f32 = rng.gen();
        direction = if direction > 90.0 { Tap::STEP_SIZE } else { -1.0 * Tap::STEP_SIZE };

        let mut user_x = rng.gen_range(Tap::MIN_X, Tap::MAX_X-Tap::USER_X_SIZE);
        
        let mut box_x = (rng.gen_range(Tap::MIN_X, Tap::MAX_X-Tap::BOX_X_SIZE) as u32) as f32;
        box_x = Tap::MIN_X;
        let mut box_size = Tap::BOX_X_SIZE;

        // place box and user box in initial positions 
        outbound_msg.send(msg::tap(user_x, box_x));

        // track if touch is causing circle radius ~ ring radius, within a given tolerance
        let mut tolerance_timer = Instant::now();
        let mut within_tolerance = false;

        // timer for time stamps outputs
        let overall_timer  = Instant::now();
        
        let mut data: Vec<world::Contacts> = vec![vec![]]; 
        let mut box_details: Vec<(f32, f32)> = vec![(box_x, box_size)];
        let mut num_boxes = 0;

        // timer to control animation FPS
        let mut animation_timer = Instant::now();

        // time animation and responses
        while overall_timer.elapsed().as_secs() < self.duration {   
            match inbound_osc.try_recv() {
                Ok((input_type, pressure, x, y, material)) => {
                    if material == self.material {
                        // update user box postion with respect to sensel input
                        user_x = range(
                            Tap::MIN_X,
                            Tap::MAX_X - Tap::USER_X_SIZE,
                            self.top_left_x,
                            self.top_left_x + self.width, 
                            x);

                        data[num_boxes].push((overall_timer.elapsed().as_millis(), pressure, x, y));

                        // is the user box inside the box?
                        if user_x >= box_x  && user_x+Tap::USER_X_SIZE <= box_x + box_size {
                            box_size = f32::max(box_size - 0.6, Tap::MIN_BOX_X_SIZE); 
                            box_x = box_x + if direction < 0.0 { -0.5 } else { 0.5 };
                            //box_x    = rng.gen_range(Tap::MIN_X, Tap::MAX_X - box_size);

                            //rng.gen_range(Tap::USER_X_SIZE, Tap::BOX_X_SIZE);
                            //box_x    = rng.gen_range(Tap::MIN_X, Tap::MAX_X - Tap::BOX_X_SIZE);
                            // update box storage
                            num_boxes = num_boxes + 1;
                            data.push(vec![]);
                            box_details.push((box_x, box_size));
                        }
                    }
                },
                _ => {}
            }

            // update box and its direction @60hz
            if animation_timer.elapsed().as_millis() > 16 {
                box_x = box_x + direction;

                // clamp within animation range and change direction of box if at boundary
                if box_x + box_size >= Tap::MAX_X  {
                    direction = -1.0 * Tap::STEP_SIZE;
                    box_x = Tap::MAX_X - box_size;
                }
                else if box_x <= Tap::MIN_X {
                    direction = Tap::STEP_SIZE;
                    box_x = Tap::MIN_X;
                }

                // reset timer
                animation_timer = Instant::now();

                // update view
                outbound_msg.send(msg::tap(user_x, box_x));
            }            
        }

        world.writeGesture("tap".to_string(), self.material, box_details, data);
    }
}


//-----------------------------------------------------------------------------
// Case when users reports properties about one or more materials
//-----------------------------------------------------------------------------

pub struct Response {
    // name of property to requested for material(s)
    name: String,
    // number of materials to be responded on
    num_materials: u32,
    // number of slide on client slide (required as there can be different response pages)
    slide_num: u32,
}

impl Response {
    pub fn new(name: String, num_materials: u32, slide_num: u32) -> Self {
        Response {
            name: name,
            num_materials: num_materials,
            slide_num: slide_num,
        }
    }
}

impl Slide for Response {
    fn run(&self, 
        world: &mut world::World,
        inbound_osc: &Receiver<msg::SenselMessage>,
        outbound_msg: &ws_server::WSServer, 
        inbound_msg:  &Receiver<msg::Message>) {

        // empty osc channel, in case of any after touches
        loop {
            match inbound_osc.try_recv() {
                Ok(_) => {
                },
                _ => { 
                    break;
                }
            }
        }

        // goto slide
        outbound_msg.send(msg::gotoSlide(self.slide_num));

        let mut num_materials = self.num_materials;
        let mut start_happened = false;

        // responses for page reported as single CSV entry
        let mut materials: Vec<String> = Vec::with_capacity(num_materials as usize);

        while num_materials > 0 {
            match inbound_osc.recv() {
                Ok((input_type, pressure, x, y, material)) => {
                    // track initial touch
                    if input_type == msg::InputType::Start {
                        start_happened = true;
                    }
                    else if input_type == msg::InputType::End && start_happened { 
                        // on release record material as response
                        materials.push(material.to_string());
                        num_materials = num_materials - 1;
                    }
                },
                _ => {},
            }
        }
        // write out response(s) to CSV
        world.writeResponse(self.name.clone(), materials);
    }
}
