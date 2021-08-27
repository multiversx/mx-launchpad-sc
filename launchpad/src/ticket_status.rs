elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq)]
pub enum TicketStatus {
    None,
    Winning { generation: u8 },
    Confirmed { generation: u8 },
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

    // Pass Option::None to ignore generation
    pub fn is_confirmed(&self, opt_current_generation: Option<u8>) -> bool {
        match *self {
            TicketStatus::Confirmed { generation } => {
                if let Some(current_generation) = opt_current_generation {
                    generation == current_generation
                } else {
                    true
                }
            }
            _ => false,
        }
    }
}
