use std::slice::IterMut;
use crate::MERKLE_MAP;
use crate::card::{Card, DEFAULT_CARDS};
use crate::config::default_local;
use crate::object::Object;
use serde::{Serialize, Serializer, ser::SerializeSeq};
use crate::Player;
use crate::StorageData;

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
    pub objects: Vec<Object>,
    pub local: Attributes,
    pub cards: Vec<Card>,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
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
    pub fn apply_object_card(&mut self, object_index: usize, counter: u64) -> Option<(usize, usize)> {
        let object = self.objects[object_index].clone();
        let current_index = object.get_modifier_index() as usize;
        if object.is_restarting() {
            let next_index = 0;
            let duration = object.cards[next_index].duration;
            let object = self.objects.get_mut(object_index).unwrap();
            object.start_new_modifier(next_index, counter);
            Some((duration as usize, next_index))
        } else {
            let applied = self.apply_modifier(&object.cards[current_index]);
            let object = self.objects.get_mut(object_index).unwrap();
            if applied {
                //zkwasm_rust_sdk::dbg!("object after: {:?}\n", object);
                //zkwasm_rust_sdk::dbg!("player after: {:?}\n", player);
                let next_index = (current_index + 1) % object.cards.len();
                let duration = object.cards[next_index].duration;
                object.start_new_modifier(next_index, counter);
                Some((duration as usize, next_index))
            } else {
                object.halt();
                None
            }
        }
    }

    pub fn restart_object_card(&mut self, object_index: usize, data: &Vec<usize>, counter: u64) -> Option<(usize, usize)> {
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
            Some((duration as usize, modifier_index as usize))
        } else {
            zkwasm_rust_sdk::dbg!("restart modifier failed, start reset modifier index... \n");
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
            objects,
            local: Attributes(local),
            cards,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
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


