use crate::StorageData;
use serde::{Serialize, Serializer};
use std::slice::IterMut;

// Custom serializer for `u64` as a string.
fn serialize_u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct Object {
    #[serde(serialize_with = "serialize_u64_as_string")]
    pub modifier_info: u64, // running << 56 + (modifier index << 48) + counter
    pub cards: [u8; 8],       // u64 contains 8 cards
    pub attributes: [u16; 4], // level, speed, efficiency, productivity
}

impl Object {
    pub fn new(cards: [u8; 8]) -> Self {
        Self {
            cards,
            modifier_info: 0,
            attributes: [0, 0, 0, 0],
        }
    }
    pub fn halt(&mut self) {
        self.modifier_info = (self.modifier_info & 0xFFFFFFFFFFFFFF) | 1 << 56;
    }

    pub fn is_halted(&self) -> bool {
        (self.modifier_info >> 56) == 1
    }

    pub fn is_restarting(&self) -> bool {
        (self.modifier_info >> 56) == 2
    }

    pub fn get_modifier_index(&self) -> u64 {
        return (self.modifier_info >> 48) & 0x7f;
    }

    pub fn start_new_modifier(&mut self, modifier_index: usize, counter: u64) {
        self.modifier_info = ((modifier_index as u64) << 48) | counter;
    }

    pub fn restart(&mut self, counter: u64) {
        self.modifier_info = (0u64 << 48) + counter;
    }

    pub fn reset_modifier(&mut self, cards: [u8; 8]) {
        self.cards = cards;
    }

    pub fn reset_halt_bit_to_restart(&mut self) {
        self.modifier_info = (self.modifier_info & 0xFFFFFFFFFFFFFF) | 1 << 57;
    }
}

impl StorageData for Object {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let modifier_info = *u64data.next().unwrap();
        let attributes = *u64data.next().unwrap();
        let card = *u64data.next().unwrap();
        Object {
            modifier_info,
            attributes: [
                (attributes & 0xff) as u16,
                ((attributes >> 16) & 0xff) as u16,
                ((attributes >> 32) & 0xff) as u16,
                ((attributes >> 48) & 0xff) as u16,
            ],
            cards: card.to_le_bytes(),
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.modifier_info);
        data.push(
            self.attributes[0] as u64
                + ((self.attributes[1] as u64) << 16)
                + ((self.attributes[2] as u64) << 32)
                + ((self.attributes[3] as u64) << 48),
        );
        data.push(u64::from_le_bytes(self.cards));
    }
}
