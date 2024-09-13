use std::collections::LinkedList;
use crate::player::AutomataPlayer;
use core::slice::IterMut;
use zkwasm_rest_abi::StorageData;

#[derive(Clone)]
pub struct Event {
    pub owner: [u64; 2],
    pub object_index: usize,
    pub delta: usize,
}

impl StorageData for Event {
    fn to_data(&self, buf: &mut Vec<u64>) {
        buf.push(self.owner[0]);
        buf.push(self.owner[1]);
        buf.push(
            ((self.object_index as u64) << 32) | self.delta as u64
        );
        zkwasm_rust_sdk::dbg!("compact {:?}", buf);
    }
    fn from_data(u64data: &mut IterMut<u64>) -> Event {
        let owner = [
            *u64data.next().unwrap(),
            *u64data.next().unwrap(),
        ];
        let f = *u64data.next().unwrap();
        Event {
            owner,
            object_index: (f >> 32) as usize,
            delta: (f & 0xffffffff) as usize,
        }
    }
}

pub struct EventQueue {
    pub counter: u64,
    pub list: std::collections::LinkedList<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            counter: 0,
            list: LinkedList::new(),
        }
    }

    pub fn dump(&self) {
        zkwasm_rust_sdk::dbg!("=-=-= dump queue =-=-=\n");
        for m in self.list.iter() {
            let delta = m.delta;
            let obj = m.object_index;
            zkwasm_rust_sdk::dbg!("[{}] - {:?}\n", delta, obj);
        }
        zkwasm_rust_sdk::dbg!("=-=-= end =-=-=\n");
    }
    pub fn tick(&mut self) {
        self.dump();
        let counter = self.counter;
        while let Some(head) = self.list.front_mut() {
            if head.delta == 0 {
                let owner_id = head.owner;
                let object_index = head.object_index;
                let mut player = AutomataPlayer::get_from_pid(&owner_id).unwrap();
                let m = if player.data.energy == 0 {
                    player.data.objects.get_mut(object_index).unwrap().halt();
                    None
                } else {
                    zkwasm_rust_sdk::dbg!("apply object card\n");
                    player.data.apply_object_card(object_index, counter)
                };
                self.list.pop_front();
                if let Some(delta) = m {
                    self.insert(object_index, &owner_id, delta);
                    if player.data.objects[object_index].get_modifier_index() == 0 {
                        player.data.energy -= 1;
                    }
                    player.store()
                }
            } else {
                head.delta -= 1;
                break;
            }
        }
        self.counter += 1;
    }

    pub fn insert(
        &mut self,
        object_index: usize,
        owner: &[u64; 2],
        delta: usize,
    ) {
        let mut delta = delta;
        let mut list = LinkedList::new();
        let mut tail = self.list.pop_front();
        while tail.is_some() && tail.as_ref().unwrap().delta <= delta {
            delta = delta - tail.as_ref().unwrap().delta;
            list.push_back(tail.unwrap());
            tail = self.list.pop_front();
        }
        let node = Event {
            object_index,
            owner: owner.clone(),
            delta,
        };
        list.push_back(node);
        match tail.as_mut() {
            Some(t) => {
                t.delta = t.delta - delta;
                list.push_back(t.clone());
            }
            None => (),
        };
        list.append(&mut self.list);
        self.list = list;
    }
}

impl StorageData for EventQueue {
    fn to_data(&self, buf: &mut Vec<u64>) {
        buf.push(self.counter);
        for e in self.list.iter() {
            e.to_data(buf);
        }
    }

    fn from_data(u64data: &mut IterMut<u64>) -> Self {
      let counter = *u64data.next().unwrap();
      let mut list = LinkedList::new();
      while u64data.len() != 0 {
          list.push_back(Event::from_data(u64data))
      }
      EventQueue {
          counter,
          list
      }
    }
}
