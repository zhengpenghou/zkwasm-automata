use crate::events::EventQueue;
use crate::settlement::SettlementInfo;
use serde::{Serialize, Serializer, ser::SerializeSeq};
use zkwasm_rest_abi::WithdrawInfo;
use zkwasm_rust_sdk::require;
use std::cell::RefCell;
use crate::player::AutomataPlayer;
use crate::player::Owner;
use crate::object::Object;
use core::slice::IterMut;
use crate::error::*;

// Custom serializer for `[u64; 4]` as a [String; 4].
fn serialize_u64_array_as_string<S>(value: &[u64; 4], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for e in value.iter() {
            seq.serialize_element(&e.to_string())?;
        }
        seq.end()
    }

pub struct SafeEventQueue(RefCell<EventQueue>);
unsafe impl Sync for SafeEventQueue {}

lazy_static::lazy_static! {
    pub static ref QUEUE: SafeEventQueue = SafeEventQueue (RefCell::new(EventQueue::new()));
}

pub struct Transaction {
    pub command: u64,
    pub objindex: usize,
    pub nonce: u64,
    pub data: Vec<u64>,
}

const INSTALL_PLAYER: u64 = 1;
const INSTALL_OBJECT: u64 = 2;
const RESTART_OBJECT: u64 = 3;
const UPGRADE_OBJECT: u64 = 4;
const INSTALL_CARD: u64 = 5;
const WITHDRAW: u64 = 6;
const DEPOSIT: u64 = 7;

impl Transaction {
    pub fn decode_error(e: u32) -> &'static str {
        match e {
           ERROR_PLAYER_NOT_EXIST => "PlayerNotExist",
           ERROR_PLAYER_ALREADY_EXIST => "PlayerAlreadyExist",
           ERROR_NOT_ENOUGH_BALANCE => "NotEnoughBalance",
           _ => "Unknown"
        }
    }
    pub fn decode(params: [u64; 4]) -> Self {
        let command = params[0] & 0xff;
        let objindex = ((params[0] >> 8) & 0xff) as usize;
        let nonce = params[0] >> 16;
        let mut data = vec![];
        if command == WITHDRAW {
            data = vec![params[1], params[2], params[3]] // address of withdraw
        } else if command == INSTALL_OBJECT || command == RESTART_OBJECT {
            for b in params[1].to_le_bytes() {
                data.push(b as u64);
            }
        } else if command == DEPOSIT {
            data = vec![params[1], params[2], params[3]] // pkey[0], pkey[1], amount
        };
        Transaction {
            command,
            objindex,
            nonce,
            data,
        }
    }
    pub fn install_player(&self, pid: &[u64; 2]) -> Result<(), u32> {
        let player = AutomataPlayer::get_from_pid(pid);
        match player {
            Some(_) => Err(ERROR_PLAYER_ALREADY_EXIST),
            None => {
                let player = AutomataPlayer::new_from_pid(*pid);
                player.store();
                Ok(())
            }
        }
    }
    pub fn install_object(&self, pid: &[u64; 2]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(self.nonce);
                let objindex = player.data.objects.len();
                unsafe {require(objindex == self.objindex)};
                player.data.pay_cost()?;
                let cards = self.data.iter().map(|x| player.data.cards[*x as usize].clone()).collect::<Vec<_>>();
                let mut object = Object::new(cards);
                let counter = QUEUE.0.borrow().counter;
                object.start_new_modifier(0, counter);
                let delay = object.cards[0].duration;
                player.data.objects.push(object);
                player.store();
                QUEUE.0.borrow_mut().insert(self.objindex, pid, delay as usize);
                Ok(()) // no error occurred
            }
        }
    }

    pub fn restart_object(&self, pid: &[u64; 2]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_ALREADY_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(self.nonce);
                player.data.pay_cost()?;
                let counter = QUEUE.0.borrow().counter;
                let data = &self.data.iter().map(|x| *x as usize).collect();
                if let Some(delay) = player.data.restart_object_card(self.objindex, data, counter) {
                    QUEUE
                        .0
                        .borrow_mut()
                        .insert(self.objindex, pid, delay);
                }
                player.store();
                Ok(())
            }
        }
    }

    pub fn upgrade_object(&self, pid: &[u64; 2], rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_ALREADY_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(self.nonce);
                player.data.pay_cost()?;
                player.data.upgrade_object(self.objindex, rand);
                player.store();
                Ok(())
            }
        }
    }


    pub fn withdraw(&self, pid: &[u64; 2]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(self.nonce);
                let amount = self.data[0] & 0xffffffff;
                player.data.cost_balance(amount as i64)?;
                let withdrawinfo = WithdrawInfo::new(&[
                    self.data[0],
                    self.data[1],
                    self.data[2]
                ]);
                SettlementInfo::append_settlement(withdrawinfo);
                player.store();
                Ok(())
            }
        }
    }

    pub fn deposit(&self, pid: &[u64; 2]) -> Result<(), u32> {
        let mut admin = AutomataPlayer::get_from_pid(pid).unwrap();
        admin.check_and_inc_nonce(self.nonce);
        let mut player = AutomataPlayer::get_from_pid(&[self.data[0], self.data[1]]);
        match player.as_mut() {
            None => {
                let mut player = AutomataPlayer::new_from_pid([self.data[0], self.data[1]]);
                player.check_and_inc_nonce(self.nonce);
                player.data.cost_balance(-(self.data[2] as i64))?;
                player.store();
            },
            Some(player) => {
                player.check_and_inc_nonce(self.nonce);
                player.data.cost_balance(-(self.data[2] as i64))?;
                player.store();
            }
        };
        Ok(()) // no error occurred
    }

    pub fn install_card(&self, pid: &[u64; 2], rand: &[u64; 4]) -> Result<(), u32> {
        let mut player = AutomataPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(self.nonce);
                let total_cards = player.data.cards.len();
                player.data.pay_cost()?;
                player.data.generate_card(rand);
                player.store();
                Ok(())
            }
        }
    }

    pub fn process(&self, pkey: &[u64; 4], rand: &[u64; 4]) -> u32 {
        let b = match self.command {
            INSTALL_PLAYER => self.install_player(&AutomataPlayer::pkey_to_pid(&pkey))
                .map_or_else(|e| e, |_| 0),
            INSTALL_OBJECT => self.install_object(&AutomataPlayer::pkey_to_pid(&pkey))
                .map_or_else(|e| e, |_| 0),
            RESTART_OBJECT => self.restart_object(&AutomataPlayer::pkey_to_pid(&pkey))
                .map_or_else(|e| e, |_| 0),
            UPGRADE_OBJECT => self.upgrade_object(&AutomataPlayer::pkey_to_pid(&pkey), rand)
                .map_or_else(|e| e, |_| 0),
            WITHDRAW => self.withdraw(&AutomataPlayer::pkey_to_pid(pkey))
                .map_or_else(|e| e, |_| 0),
            INSTALL_CARD => self.install_card(&AutomataPlayer::pkey_to_pid(pkey), rand)
                .map_or_else(|e| e, |_| 0),
            DEPOSIT => self.deposit(&AutomataPlayer::pkey_to_pid(pkey))
                .map_or_else(|e| e, |_| 0),
            _ => {
                QUEUE.0.borrow_mut().tick();
                0
            }
        };
        b
    }

    pub fn automaton() {
        QUEUE.0.borrow_mut().tick();
    }
}

pub struct State {}

impl State {
    pub fn get_state(pid: Vec<u64>) -> String {
        let player = AutomataPlayer::get(&pid.try_into().unwrap()).unwrap();
        let counter = QUEUE.0.borrow().counter;
        serde_json::to_string(&(player, counter)).unwrap()
    }

    pub fn preempt() -> bool {
        let counter = QUEUE.0.borrow().counter;
        if counter % 30 == 0 {
            true
        } else {
            false
        }
    }

    pub fn flush_settlement() -> Vec<u8> {
        SettlementInfo::flush_settlement()
    }

    pub fn rand_seed() -> u64 {
        0
    }

    pub fn store() {
        QUEUE.0.borrow_mut().store();
    }
    pub fn initialize() {
        QUEUE.0.borrow_mut().fetch();
    }
}
