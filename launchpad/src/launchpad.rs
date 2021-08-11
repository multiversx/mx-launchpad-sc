#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::derive::contract]
pub trait Launchpad {
    #[init]
    fn init(&self) {}
}
