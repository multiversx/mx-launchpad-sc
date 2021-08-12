elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq)]
pub enum TicketStatus {
    None,
    Normal,
    Winning,
    Confirmed,
    Redeemed 
}
