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

const LOCAL_RESOURCE_WEIGHT: [u64; LOCAL_ATTRIBUTES_SIZE] = [1, 1, 2, 4, 4, 32, 64, 512];
pub const COST_INCREASE_ROUND: u32 = 5;

pub fn random_modifier(current_resource: [u64; LOCAL_ATTRIBUTES_SIZE], rand: u64) -> Card {
    todo!()
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
