elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const MIN_GAS_TO_SAVE_PROGRESS: u64 = 150_000 + 50_000 * 20; // base_cost + 50K per bytes of key and value

#[derive(TopDecode, TopEncode, TypeAbi, PartialEq)]
pub enum OngoingOperationType {
    None,
    AddTickets {
        index: usize
    }
}

#[elrond_wasm::module]
pub trait OngoingOperationModule {
    fn can_continue_operation(&self, operation_cost: u64) -> bool {
        let gas_left = self.blockchain().get_gas_left();

        gas_left - operation_cost > MIN_GAS_TO_SAVE_PROGRESS
    }

    #[storage_mapper("operation")]
    fn current_ongoing_operation(&self) -> SingleValueMapper<Self::Storage, OngoingOperationType>;
}
