use std::collections::LinkedList;
use crate::player::AutomataPlayer;

#[derive(Clone)]
pub struct Event {
    pub owner: [u64; 2],
    pub object_index: usize,
    pub delta: usize,
}

impl Event {
    fn compact(&self, buf: &mut Vec<u64>) {
        buf.push(self.owner[0]);
        buf.push(self.owner[1]);
        buf.push(
            ((self.object_index as u64) << 32) | self.delta as u64
        );
        zkwasm_rust_sdk::dbg!("compact {:?}", buf);
    }
    fn fetch(buf: &mut Vec<u64>) -> Event {
        zkwasm_rust_sdk::dbg!("fetch{:?}", buf);
        let f = buf.pop().unwrap();
        let mut owner = [
            buf.pop().unwrap(),
            buf.pop().unwrap(),
        ];
        owner.reverse();
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
    pub fn store(&self, buf: &mut Vec<u64>) {
        for e in self.list.iter() {
            e.compact(buf);
        }
        buf.push(self.counter);
    }
    pub fn fetch(&mut self, data: &mut Vec<u64>) {
        if !data.is_empty() {
            let counter = data.pop().unwrap();
            let mut list = LinkedList::new();
            while !data.is_empty() {
                list.push_back(Event::fetch(data))
            }
            self.counter = counter;
            self.list = list;
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
                    player.data.apply_object_card(object_index, counter)
                };
                self.list.pop_front();
                if let Some(delta) = m {
                    self.insert(object_index, &owner_id, delta);
                    if player.data.objects[object_index].get_modifier_index() == 0 {
                        player.data.energy -= 1;
                    }
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
