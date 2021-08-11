#![no_std]

elrond_wasm::imports!();

mod random;
use random::*;

#[elrond_wasm::derive::contract]
pub trait Launchpad {
    #[init]
    fn init(&self) {}
}
