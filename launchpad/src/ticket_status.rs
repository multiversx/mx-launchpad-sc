elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq)]
pub enum TicketStatus {
    None,
    Winning { generation: u8 },
    Confirmed,
    Redeemed,
}

impl TicketStatus {
    pub fn is_winning(&self, current_generation: u8) -> bool {
        if let TicketStatus::Winning { generation } = *self {
            if generation == current_generation {
                return true;
            }
        }

        false
    }

    pub fn is_confirmed(&self) -> bool {
        matches!(*self, TicketStatus::Confirmed)
    }
}
