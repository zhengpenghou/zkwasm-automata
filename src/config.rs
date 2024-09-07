use crate::card::Card;
use serde::Serialize;
pub const ENTITY_ATTRIBUTES_SIZE: usize = 4; //level speed efficiency productivity
pub const LOCAL_ATTRIBUTES_SIZE: usize = 8;

#[derive(Serialize, Clone)]
pub struct Config {
    version: &'static str,
    entity_attributes: [&'static str; ENTITY_ATTRIBUTES_SIZE],
    local_attributes: [&'static str; LOCAL_ATTRIBUTES_SIZE],
    object_cost_exp: u64, // 0, 0, 1, 2, 4, 8, ....  index < 2: 0, index >=2:  cost_exp ^(level-2)
    upgrade_cost_exp: u64, // 1, 2, 4, 8 ...
}

pub fn default_entities(index: usize) -> [i64; ENTITY_ATTRIBUTES_SIZE] {
    if index < 2 {
        [0, 0, 0, 0]
    } else {
        let mut v = [0, 0, 0, 0];
        for i in 0..index-2 {
            v[i%3 + 1] += 1
        }
        v
    }
}

pub fn default_local() -> [i64; LOCAL_ATTRIBUTES_SIZE] {
    [30, 30, 0, 0, 2, 0, 0, 0]
}

const LOCAL_RESOURCE_WEIGHT: [u64; LOCAL_ATTRIBUTES_SIZE] = [1, 1, 2, 4, 8, 16, 32, 128];
pub const COST_INCREASE_ROUND: u32 = 5;

pub fn random_modifier(current_resource: [i64; LOCAL_ATTRIBUTES_SIZE], rand: u64) -> Card {
    let rand_bytes = rand.to_le_bytes().map(|x| x as u64);
    let output1 = rand_bytes[0] & 0x7; // select two target result
    let output2 = (rand_bytes[0] >> 4) & 0x7; // select two target result
    let cost1 = (rand_bytes[1] & 0x3) as u64; // select two target number
    let cost2 = ((rand_bytes[1] >> 4) & 0x3) as u64; // select two target number
    let mut weight = output1 * cost2 * LOCAL_RESOURCE_WEIGHT[output1 as usize];
    weight += output2 * cost2 * LOCAL_RESOURCE_WEIGHT[output2 as usize];
    let input1 = (rand_bytes[2] & 0x7) as usize;
    let input2 = ((rand_bytes[2] >> 4) & 0x7) as usize;
    let input3 = (rand_bytes[3] & 0x7) as usize;
    let input4 = ((rand_bytes[3] >> 4) & 0x7) as usize;
    let mut inputs = [input1, input2, input3, input4];
    inputs.sort();

    let distribute = current_resource[input1]
        + current_resource[input2]
        + current_resource[input3]
        + current_resource[input4];
    let cost = inputs.map(
        |x| weight * (distribute as u64) / (LOCAL_RESOURCE_WEIGHT[x] + (current_resource[x] as u64)) + 1);
    let mut attrs = [0i64; 8];
    attrs[inputs[0] as usize] += cost[0] as i64;
    attrs[inputs[1] as usize] += cost[1] as i64;
    attrs[inputs[2] as usize] += cost[2] as i64;
    attrs[inputs[3] as usize] += cost[3] as i64;
    attrs[output1 as usize ] -= cost1 as i64;
    attrs[output2 as usize] -= cost2 as i64;
    let attrs = attrs.map(|x| {
        if x > 64 {
            64 as i8
        } else if x < -64 {
            -64 as i8
        } else {
            x as i8
        }
    });

    let mut weight = 0i64;
    for i in 0..LOCAL_ATTRIBUTES_SIZE {
        weight = weight + (attrs[i] as i64) * (LOCAL_RESOURCE_WEIGHT[i] as i64)
    }
    weight += 5;
    zkwasm_rust_sdk::dbg!("random modifier weight {}\n", weight);

    let duration = if weight > 0 {
        15
    } else {
        weight * 40 + 15
    };

    Card {
        duration: duration as u64,
        attributes: attrs
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config {
        version: "1.0",
        object_cost_exp: 2,
        upgrade_cost_exp: 2,
        entity_attributes: ["Level", "Speed", "Efficiency", "Producitivity"],
        local_attributes: ["Engery Crystal", "Instellar Mineral", "Biomass", "Quantum Foam", "Necrodermis", "Alien Floral", "Spice Melange", "Titanium"],
    };
}

impl Config {
    pub fn to_json_string() -> String {
        serde_json::to_string(&CONFIG.clone()).unwrap()
    }
    pub fn autotick() -> bool {
        true
    }
}
