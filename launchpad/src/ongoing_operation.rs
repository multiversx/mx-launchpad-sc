elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const MIN_GAS_TO_SAVE_PROGRESS: u64 = 25_000_000;

#[derive(TopDecode, TopEncode, TypeAbi, PartialEq)]
pub enum OngoingOperationType {
    None,
    AddTickets {
        index: usize,
    },
    SelectWinners {
        seed: H256,
        seed_index: usize,
        ticket_position: usize,
    },
    RestartConfirmationPeriod {
        ticket_position: usize,
    },
}

#[elrond_wasm::module]
pub trait OngoingOperationModule {
    fn can_continue_operation(&self, operation_cost: u64) -> bool {
        let gas_left = self.blockchain().get_gas_left();

        gas_left - operation_cost > MIN_GAS_TO_SAVE_PROGRESS
    }

    fn save_progress(&self, operation: &OngoingOperationType) {
        self.current_ongoing_operation().set(operation);
    }

    fn clear_operation(&self) {
        self.current_ongoing_operation().clear();
    }

    fn require_no_ongoing_operation(&self) -> SCResult<()> {
        require!(
            self.current_ongoing_operation().is_empty(),
            "Another ongoing operation is in progress"
        );
        Ok(())
    }

    #[storage_mapper("operation")]
    fn current_ongoing_operation(&self) -> SingleValueMapper<Self::Storage, OngoingOperationType>;
}
