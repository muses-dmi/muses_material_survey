//! Protocol Messages as JSON    
//! 
//! Copyright: Benedict R. Gaster
//! 
//! 

use serde::Deserialize;
use serde_json::json;
use serde_json::{Value};

//-----------------------------------------------------------------------
// Message type
//-----------------------------------------------------------------------

pub type Message = serde_json::Value; 

/// Input type of touch on sensel, i.e start touch, move, and end touch
#[derive(PartialEq,Debug)]
pub enum InputType {
    Start,
    Move,
    End,
    None
}

impl InputType {
    pub fn new(num: u32) -> InputType {
        if num == 0 {
            return InputType::Start;
        }
        else if num == 1 {
            return InputType::Move;
        }
        else if num == 2 {
            return InputType::End;
        }
        
        return InputType::None;
    }
}

// (InputType, pressure, x, y, material)
pub type SenselMessage = (InputType, f32, f32, f32, u32);

//-----------------------------------------------------------------------
// Client to Server
//-----------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct Likert {
    pub name: String,
    pub value: u32, 
}

pub fn likert(data: Message) -> Result<Likert, ()> {
    match data {
        Value::Object(obj) => {
            if obj.contains_key("type") {
                let s : String = serde_json::from_value(obj["type"].clone()).unwrap();
                if s == "likert" {
                    return Ok(Likert {
                        name: serde_json::from_value(obj["name"].clone()).unwrap(),
                        value: serde_json::from_value(obj["value"].clone()).unwrap(),
                    });
                }
            }
        },
        _ => {

        }
    }

    return Err(());
}

pub fn is_type(name: &str, data: Message) -> bool {
    //let v: serde_json::Result<Value> = serde_json::from_str(&data);
    match data {
        Value::Object(obj) => {
            if obj.contains_key("type") {
                let s : String = serde_json::from_value(obj["type"].clone()).unwrap();
                if s == name {
                    return true;
                }
            }
        }
        _ => {
        }
    }
    false
}

pub fn is_begin(data: Message) -> bool {
    is_type("begin", data)
}

pub fn is_consent(data: Message) -> bool {
    is_type("consent", data)
}

pub fn is_connected(data: Message) -> bool {
    is_type("connected", data)
}

//-----------------------------------------------------------------------
// Server to client
//-----------------------------------------------------------------------

pub fn consentID(id: String) -> Message {
    json!({
        "type": "consentID",
        "id": id 
    })
}

pub fn press(circle: f32, ring: f32) -> Message {
    json!({
        "type": "press",
        "circle": circle,
        "ring": ring 
    })
}

pub fn slider(user_x: f32, box_x: f32, box_size: f32) -> Message {
    json!({
        "type": "slider",
        "user_x": user_x,
        "box_x": box_x,
        "box_size": box_size
    })
}

pub fn tap(user_x: f32, arrow_x: f32) -> Message {
    json!({
        "type": "tap",
        "user_x": user_x,
        "arrow_x": arrow_x
    })
}

pub fn materialIndex(index: u32, slide: u32) -> Message {
    json!({
        "type": "materialIndex",
        "slide": slide,
        "value": index 
    })
}

pub fn gestureType(value: String) -> Message {
    json!({
        "type": "gestureType",
        "value": value 
    })
}

pub fn gotoSlide(slide: u32) -> Message {
    json!({
        "type": "goto",
        "slide": slide 
    })
}

pub fn gotoFrontMatter() -> Message {
    gotoSlide(0)
}

pub fn gotoConsent() -> Message {
    gotoSlide(1)
}

pub fn gotoLikert() -> Message {
    gotoSlide(2)
}

pub fn gotoPress() -> Message {
    gotoSlide(3)
}

pub fn gotoMostAccurate() -> Message {
    gotoSlide(most_accurate_num())
}

pub fn gotoMostComfortable() -> Message {
    gotoSlide(most_comfortable_num())
}

pub fn gotoMostResponsive() -> Message {
    gotoSlide(most_responsive_num())
}

pub fn gotoOrderFavorite() -> Message {
    gotoSlide(order_favorite_num())
}

pub fn gotoSlider() -> Message {
    gotoSlide(slider_num())
}

pub fn gotoTap() -> Message {
    gotoSlide(tap_num())
}

pub fn front_matter_num() -> u32 {
    0
}

pub fn consent_num() -> u32 {
    1
}

pub fn likert_num() -> u32 {
    2
}

pub fn press_num() -> u32 {
    3
}

pub fn most_accurate_num() -> u32 {
    4
}

pub fn most_comfortable_num() -> u32 {
    5
}

pub fn most_responsive_num() -> u32 {
    6
}

pub fn order_favorite_num() -> u32 {
    7
}

pub fn slider_num() -> u32 {
    8
}

pub fn tap_num() -> u32 {
    9
}

