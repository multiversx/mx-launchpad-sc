elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::{random::Random, FIRST_TICKET_ID};

const MIN_GAS_TO_SAVE_PROGRESS: u64 = 10_000_000;
static ANOTHER_OP_ERR_MSG: &[u8] = b"Another ongoing operation is in progress";

#[derive(TypeAbi, TopEncode, TopDecode)]
pub enum OngoingOperationType<M: ManagedTypeApi + CryptoApi> {
    None,
    FilterTickets {
        first_ticket_id_in_batch: usize,
        nr_removed: usize,
    },
    SelectWinners {
        rng: Random<M>,
        ticket_position: usize,
    },
    AdditionalSelection {
        encoded_data: ManagedBuffer<M>,
    },
}

pub type LoopOp = bool;
pub const CONTINUE_OP: bool = true;
pub const STOP_OP: bool = false;

#[elrond_wasm::module]
pub trait OngoingOperationModule {
    fn run_while_it_has_gas<Process>(&self, mut process: Process) -> OperationCompletionStatus
    where
        Process: FnMut() -> LoopOp,
    {
        let gas_before = self.blockchain().get_gas_left();

        let mut loop_op = process();

        let gas_after = self.blockchain().get_gas_left();
        let gas_per_iteration = gas_before - gas_after;

        loop {
            if loop_op == STOP_OP {
                break;
            }

            if !self.can_continue_operation(gas_per_iteration) {
                return OperationCompletionStatus::InterruptedBeforeOutOfGas;
            }

            loop_op = process();
        }

        self.clear_operation();

        OperationCompletionStatus::Completed
    }

    fn can_continue_operation(&self, operation_cost: u64) -> bool {
        let gas_left = self.blockchain().get_gas_left();

        gas_left > MIN_GAS_TO_SAVE_PROGRESS + operation_cost
    }

    #[inline]
    fn save_progress(&self, op: &OngoingOperationType<Self::Api>) {
        self.current_ongoing_operation().set(op);
    }

    #[inline]
    fn clear_operation(&self) {
        self.current_ongoing_operation().clear();
    }

    fn load_filter_tickets_operation(&self) -> (usize, usize) {
        let ongoing_operation = self.current_ongoing_operation().get();
        match ongoing_operation {
            OngoingOperationType::None => (FIRST_TICKET_ID, 0),
            OngoingOperationType::FilterTickets {
                first_ticket_id_in_batch,
                nr_removed,
            } => (first_ticket_id_in_batch, nr_removed),
            _ => sc_panic!(ANOTHER_OP_ERR_MSG),
        }
    }

    fn load_select_winners_operation(&self) -> (Random<Self::Api>, usize) {
        let ongoing_operation = self.current_ongoing_operation().get();
        match ongoing_operation {
            OngoingOperationType::None => (Random::default(), FIRST_TICKET_ID),
            OngoingOperationType::SelectWinners {
                rng,
                ticket_position,
            } => (rng, ticket_position),
            _ => sc_panic!(ANOTHER_OP_ERR_MSG),
        }
    }

    fn load_additional_selection_operation<T: TopDecode + Default>(&self) -> T {
        let ongoing_operation = self.current_ongoing_operation().get();
        match ongoing_operation {
            OngoingOperationType::None => T::default(),
            OngoingOperationType::AdditionalSelection { encoded_data } => {
                T::top_decode(encoded_data)
                    .unwrap_or_else(|_| sc_panic!("Failed to deserialize custom ongoing operation"))
            }
            _ => sc_panic!(ANOTHER_OP_ERR_MSG),
        }
    }

    #[storage_mapper("operation")]
    fn current_ongoing_operation(&self) -> SingleValueMapper<OngoingOperationType<Self::Api>>;
}
