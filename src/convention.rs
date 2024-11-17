use core::slice::IterMut;
use std::collections::LinkedList;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_abi::MERKLE_MAP;
pub trait EventHandler: Clone + StorageData {
    fn get_delta(&self) -> usize;
    fn progress(&mut self, d: usize);
    fn handle(&mut self, counter: u64) -> Option<Self>;
    fn u64size() -> usize;
}

pub struct EventQueue<T: EventHandler + Sized> {
    pub counter: u64,
    pub list: std::collections::LinkedList<T>,
}

impl<E: EventHandler> EventQueue<E> {
    pub fn new() -> Self {
        EventQueue {
            counter: 0,
            list: LinkedList::new(),
        }
    }

    pub fn dump(&self, counter: u64) {
        zkwasm_rust_sdk::dbg!("dump queue: {}", counter);
        for m in self.list.iter() {
            let delta = m.get_delta();
            zkwasm_rust_sdk::dbg!(" {}", delta);
        }
        zkwasm_rust_sdk::dbg!("\n");
    }
    pub fn tick(&mut self) {
        let counter = self.counter;
        self.dump(counter);
        let mut entries_data = self.get_old_entries(counter);
        let entries_nb = entries_data.len() / E::u64size();
        let mut dataiter = entries_data.iter_mut();
        let mut entries = Vec::with_capacity(entries_nb);
        for _ in 0..entries_nb {
            entries.push(E::from_data(&mut dataiter));
        }
        zkwasm_rust_sdk::dbg!("entries from storage: {} at counter {}\n", entries_nb, {
            self.counter
        });
        // perform activities from existing entries
        for mut e in entries {
            let m = e.handle(counter);

            if let Some(event) = m {
                self.insert(event);
            }
        }

        while let Some(head) = self.list.front_mut() {
            if head.get_delta() == 0 {
                let m = head.handle(counter);
                self.list.pop_front();
                if let Some(event) = m {
                    self.insert(event);
                }
            } else {
                head.progress(1);
                break;
            }
        }
        self.counter += 1;
    }

    pub fn insert(&mut self, node: E) {
        let mut event = node.clone();
        let mut cursor = self.list.cursor_front_mut();
        while cursor.current().is_some()
            && cursor.current().as_ref().unwrap().get_delta() <= event.get_delta()
        {
            event.progress(cursor.current().as_ref().unwrap().get_delta());
            cursor.move_next();
        }
        match cursor.current() {
            Some(t) => {
                t.progress(event.get_delta());
            }
            None => (),
        };

        cursor.insert_before(event);
    }
}

impl<T: EventHandler + Sized> StorageData for EventQueue<T> {
    fn to_data(&self, buf: &mut Vec<u64>) {
        buf.push(self.counter);
    }

    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let counter = *u64data.next().unwrap();
        let list = LinkedList::new();
        EventQueue { counter, list }
    }
}

const EVENTS_LEAF_INDEX: u64 = 0xfffffffe;

impl<T: EventHandler + Sized> EventQueue<T> {
    fn get_old_entries(&self, counter: u64) -> Vec<u64> {
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.get(&[EVENTS_LEAF_INDEX, counter & 0xfffffff, 0, EVENTS_LEAF_INDEX])
    }
    fn set_entries(&self, entries: &Vec<u64>, counter: u64) {
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(
            &[EVENTS_LEAF_INDEX, counter & 0xfffffff, 0, EVENTS_LEAF_INDEX],
            entries.as_slice(),
        );
        zkwasm_rust_sdk::dbg!("store {} entries at counter {}", { entries.len() }, counter);
    }
    pub fn store(&mut self) {
        let mut tail = self.list.pop_front();
        let mut store = vec![];
        let mut current_delta = 0u64;
        while tail.is_some() {
            let delta = tail.as_ref().unwrap().get_delta();
            if delta as u64 > 0 {
                if !store.is_empty() {
                    let mut entries = self.get_old_entries(current_delta + self.counter);
                    entries.append(&mut store);
                    self.set_entries(&entries, current_delta + self.counter);
                    store.clear();
                }
            }
            current_delta += delta as u64;
            tail.as_ref().unwrap().to_data(&mut store);
            tail = self.list.pop_front();
        }
        if !store.is_empty() {
            let mut entries = self.get_old_entries(current_delta + self.counter);
            entries.append(&mut store);
            self.set_entries(&entries, current_delta + self.counter);
            store.clear();
        }
    }
}
