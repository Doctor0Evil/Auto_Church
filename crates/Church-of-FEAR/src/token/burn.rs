use crate::ledger::deed_event::DeedEvent;

pub fn burn_for_harm(current_balance: u64, event: &DeedEvent) -> u64 {
    if event.life_harm_flag {
        current_balance / 2
    } else {
        current_balance
    }
}
