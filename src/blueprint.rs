use std::{collections::HashMap, io::prelude::*};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use serde::{Serialize, Deserialize};

pub fn read_blueprint(blueprint: &str) -> Blueprint {
    assert_eq!(blueprint.chars().next(),Some('0'),"bad version");
    let (_,b64) = blueprint.split_at(1);
    let data = base64::decode(b64 ).expect("bad base64");

    let mut decomp = ZlibDecoder::new(&*data);
    let mut json = String::new();
    decomp.read_to_string(&mut json).unwrap();
    println!("=> {}",json);
    
    let wrapper: BlueprintWrapper = serde_json::from_str(&json).expect("bad json");
    wrapper.blueprint
}

pub fn write_blueprint(blueprint: Blueprint) -> String {
    let wrapper = BlueprintWrapper{blueprint};
    let json = serde_json::to_string(&wrapper).expect("serialize failed");
    //println!("=> {}",json);

    let mut compress = ZlibEncoder::new(Vec::new(), Compression::best());
    compress.write_all(json.as_bytes()).unwrap();
    let data = compress.finish().unwrap();
    let b64 = base64::encode(data);
    format!("0{}",b64)
}

#[derive(Debug,Serialize,Deserialize)]
struct BlueprintWrapper {
    blueprint: Blueprint
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Blueprint {
    pub entities: Vec<Entity>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Entity {
    pub entity_number: u32,
    pub name: String,
    pub position: Position,
    pub direction: u32, // usually 4 for us
    pub control_behavior: ControlBehavior,
    pub connections: Option<HashMap<u32,Connections>> // key = circuit id
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ControlBehavior {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arithmetic_conditions: Option<ArithmeticConditions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decider_conditions: Option<DeciderConditions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<Filter>>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ArithmeticConditions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_signal: Option<Signal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_constant: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_signal: Option<Signal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_constant: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_signal: Option<Signal>,
    pub operation: String
}

#[derive(Debug,Serialize,Deserialize)]
pub struct DeciderConditions {
    pub first_signal: Signal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_signal: Option<Signal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_signal: Option<Signal>,
    pub comparator: String,
    pub copy_count_from_input: bool
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Signal {
    #[serde(rename = "type")]
    pub cat: String,
    pub name: String
}

#[derive(Debug,Serialize,Deserialize,Default)]
pub struct Connections {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub red: Option<Vec<Connection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub green: Option<Vec<Connection>>,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Connection {
    pub entity_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_id: Option<u32> // input = 1, output = 2
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Filter {
    pub signal: Signal,
    pub count: i32,
    pub index: u32
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Position{
    pub x: f32,
    pub y: f32
}
