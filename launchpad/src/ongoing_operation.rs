elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::{random::Random, FIRST_TICKET_ID};

const MIN_GAS_TO_SAVE_PROGRESS: u64 = 10_000_000;

#[derive(TypeAbi, TopEncode, TopDecode)]
pub enum OngoingOperationType {
    None,
    FilterTickets {
        first_ticket_id_in_batch: usize,
        nr_removed: usize,
    },
    SelectWinners {
        seed: crate::random::Hash,
        seed_index: usize,
        ticket_position: usize,
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

    #[inline(always)]
    fn can_continue_operation(&self, operation_cost: u64) -> bool {
        let gas_left = self.blockchain().get_gas_left();

        gas_left > MIN_GAS_TO_SAVE_PROGRESS + operation_cost
    }

    #[inline(always)]
    fn save_progress(&self, op: &OngoingOperationType) {
        self.current_ongoing_operation().set(op);
    }

    #[inline(always)]
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
            _ => sc_panic!("Another ongoing operation is in progress"),
        }
    }

    fn load_select_winners_operation(&self) -> (Random, usize) {
        let ongoing_operation = self.current_ongoing_operation().get();
        match ongoing_operation {
            OngoingOperationType::None => (Random::new(), FIRST_TICKET_ID),
            OngoingOperationType::SelectWinners {
                seed,
                seed_index,
                ticket_position,
            } => (Random::from_hash(seed, seed_index), ticket_position),
            _ => sc_panic!("Another ongoing operation is in progress"),
        }
    }

    #[storage_mapper("operation")]
    fn current_ongoing_operation(&self) -> SingleValueMapper<OngoingOperationType>;
}
