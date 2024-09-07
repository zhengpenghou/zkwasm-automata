use std::slice::IterMut;
use crate::MERKLE_MAP;
use crate::card::{Card, DEFAULT_CARDS};
use crate::config::{default_local, random_modifier};
use crate::object::Object;
use serde::Serialize;
use crate::Player;
use crate::StorageData;
use crate::config::COST_INCREASE_ROUND;
use crate::error::ERROR_NOT_ENOUGH_BALANCE;

#[derive(Clone, Debug, Serialize)]
pub struct Attributes(pub Vec<i64>);

impl Attributes {
    pub fn apply_modifier(&mut self, m: &Attributes) -> bool {
        for (a, b) in self.0.iter().zip(m.0.iter()) {
            if *a + *b < 0 {
                return false;
            }
        }
        for (a, b) in self.0.iter_mut().zip(m.0.iter()) {
            *a += *b;
        }
        return true;
    }
}

#[derive(Debug, Serialize)]
pub struct PlayerData {
    pub cost_info: u32,
    pub current_cost: u32,
    pub objects: Vec<Object>,
    pub local: Attributes,
    pub cards: Vec<Card>,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            cost_info: COST_INCREASE_ROUND,
            current_cost: 0,
            objects: vec![],
            local: Attributes::default_local(),
            cards: DEFAULT_CARDS.clone(),
        }
    }
}

impl Attributes {
    fn default_local() -> Self {
        Attributes(default_local().to_vec())
    }
}

impl PlayerData {
    pub fn generate_card(&mut self, rand: &[u64; 4]) {
        let new_card = random_modifier(self.local.0.clone().try_into().unwrap(), rand[1]);
        self.cards.push(new_card)
    }

    pub fn pay_cost(&mut self) -> Result <(), u32> {
        self.cost_balance(self.current_cost as i64)?;
        self.cost_info -= 1;
        if self.cost_info == 0 {
            self.cost_info = COST_INCREASE_ROUND;
            if self.current_cost != 0 {
                self.current_cost = self.current_cost * 2
            } else {
                self.current_cost = 1;
            }
        }
        Ok(())
    }

    pub fn cost_balance(&mut self, b: i64) -> Result <(), u32> {
        if let Some(treasure) = self.local.0.last_mut() {
            if *treasure >= b {
                *treasure -= b;
                Ok(())
            } else {
                Err(ERROR_NOT_ENOUGH_BALANCE)
            }
        } else {
            unreachable!();
        }
    }

    pub fn upgrade_object(&mut self, object_index: usize, rand: &[u64; 4]) {
        let mode = (rand[2] % 3) as usize;
        let object = self.objects.get_mut(object_index).unwrap();
        unsafe { zkwasm_rust_sdk::require(object.attributes[0]<128) };
        object.attributes[0] += 1;
        object.attributes[mode] += 1;
    }

    pub fn apply_object_card(&mut self, object_index: usize, counter: u64) -> Option<usize> {
        let object = self.objects[object_index].clone();
        let current_index = object.get_modifier_index() as usize;
        if object.is_restarting() {
            let next_index = 0;
            let duration = object.cards[next_index].duration;
            let object = self.objects.get_mut(object_index).unwrap();
            object.start_new_modifier(next_index, counter);
            Some(duration as usize)
        } else {
            let applied = self.apply_modifier(&object.cards[current_index]);
            let object = self.objects.get_mut(object_index).unwrap();
            if applied {
                //zkwasm_rust_sdk::dbg!("object after: {:?}\n", object);
                //zkwasm_rust_sdk::dbg!("player after: {:?}\n", player);
                let next_index = (current_index + 1) % object.cards.len();
                let duration = object.cards[next_index].duration;
                object.start_new_modifier(next_index, counter);
                Some(duration as usize)
            } else {
                object.halt();
                None
            }
        }
    }

    pub fn restart_object_card(&mut self, object_index: usize, data: &Vec<usize>, counter: u64) -> Option<usize> {
        let object = self.objects.get_mut(object_index).unwrap();
        let halted = object.is_halted();
        if halted {
            // modify object with new modifiers
            let cards = data.iter().map(|x| self.cards[*x].clone()).collect::<Vec<_>>();
            object.reset_modifier(cards);
            let modifier_index = object.get_modifier_index();
            let duration = object.cards[modifier_index as usize].duration;
            object.restart(counter);
            zkwasm_rust_sdk::dbg!("object restarted\n");
            Some(duration as usize)
        } else {
            object.reset_halt_bit_to_restart();
            None
        }
    }
    pub fn apply_modifier(&mut self, m: &Card) -> bool {
        let m = m.attributes.iter().map(|x| *x as i64).collect::<Vec<_>>();
        for (a, b) in self.local.0.iter().zip(m.iter()) {
            if *a + *b < 0 {
                return false;
            }
        }
        for (a, b) in self.local.0.iter_mut().zip(m.iter()) {
            *a += *b;
        }
        return true;
    }
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let cost_info = *u64data.next().unwrap();
        let objects_size = *u64data.next().unwrap();
        let mut objects = Vec::with_capacity(objects_size as usize);
        for _ in 0..objects_size {
            objects.push(Object::from_data(u64data));
        }

        let local_size = *u64data.next().unwrap();
        let mut local = Vec::with_capacity(local_size as usize);
        for _ in 0..local_size {
            local.push(*u64data.next().unwrap() as i64);
        }

        let card_size = *u64data.next().unwrap();
        let mut cards = Vec::with_capacity(card_size as usize);
        for _ in 0..card_size {
            cards.push(Card::from_data(u64data));
        }
        PlayerData {
            cost_info: (cost_info >> 32) as u32,
            current_cost: (cost_info & 0xffffffff) as u32,
            objects,
            local: Attributes(local),
            cards,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(((self.cost_info as u64) << 32) + (self.current_cost as u64));
        data.push(self.objects.len() as u64);
        for c in self.objects.iter() {
            c.to_data(data);
        }
        data.push(self.local.0.len() as u64);
        for c in self.local.0.iter() {
            data.push(*c as u64);
        }
        data.push(self.cards.len() as u64);
        for c in self.cards.iter() {
            c.to_data(data);
        }
    }
}

pub type AutomataPlayer = Player<PlayerData>;

pub trait Owner: Sized {
    fn store(&self);
    fn new(pkey: &[u64; 4]) -> Self;
    fn get(pkey: &[u64; 4]) -> Option<Self>;
}

impl Owner for AutomataPlayer {
    fn store(&self) {
        zkwasm_rust_sdk::dbg!("store player\n");
        let mut data = Vec::new();
        self.data.to_data(&mut data);
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&Self::to_key(&self.player_id), data.as_slice());
        zkwasm_rust_sdk::dbg!("end store player\n");
    }
    fn new(pkey: &[u64; 4]) -> Self {
        Self::new_from_pid(Self::pkey_to_pid(pkey))
    }

    fn get(pkey: &[u64; 4]) -> Option<Self> {
        Self::get_from_pid(&Self::pkey_to_pid(pkey))
    }
}


