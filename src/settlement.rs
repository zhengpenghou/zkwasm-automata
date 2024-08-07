use crate::state::State;
use zkwasm_rest_abi::{WithdrawInfo, MERKLE_MAP};
pub struct SettlementInfo(Vec<WithdrawInfo>);

const WITHDRAW_OPCODE:[u8; 8] = [1, 0, 0, 0, 0, 0, 0, 0];

pub static mut SETTLEMENT: SettlementInfo = SettlementInfo(vec![]);

impl SettlementInfo {
    pub fn append_settlement(info: WithdrawInfo) {
        unsafe { SETTLEMENT.0.push(info) };
    }
    pub fn flush_settlement() -> Vec<u8> {
        zkwasm_rust_sdk::dbg!("flush settlement\n");
        let sinfo = unsafe { &mut SETTLEMENT };
        let mut bytes: Vec<u8> = Vec::with_capacity(sinfo.0.len() * 32);
        for s in &sinfo.0 {
            s.flush(&mut bytes);
        }
        sinfo.0 = vec![];
        State::store();
        bytes
    }
}
