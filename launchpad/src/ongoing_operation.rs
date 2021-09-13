elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::{random::Random, VEC_MAPPER_START_INDEX};

const MIN_GAS_TO_SAVE_PROGRESS: u64 = 100_000_000;

#[derive(TopDecode, TopEncode, TypeAbi, PartialEq)]
pub enum OngoingOperationType {
    None,
    SelectWinners {
        seed: H256,
        seed_index: usize,
        ticket_position: usize,
        nr_winning_tickets: usize,
    },
    SelectNewWinners {
        ticket_position: usize,
        nr_winning_tickets: usize,
    },
}

pub enum LoopOp {
    Continue,
    Save(OngoingOperationType),
    Break,
}

impl LoopOp {
    fn is_break(&self) -> bool {
        return matches!(self, LoopOp::Break);
    }
}

#[elrond_wasm::module]
pub trait OngoingOperationModule: crate::setup::SetupModule {
    fn run_while_it_has_gas<Process>(
        &self,
        mut process: Process,
    ) -> SCResult<OperationCompletionStatus>
    where
        Process: FnMut() -> LoopOp,
    {
        let gas_before = self.blockchain().get_gas_left();

        let mut loop_op = process();

        let gas_after = self.blockchain().get_gas_left();
        let gas_per_iteration = gas_before - gas_after;

        loop {
            if loop_op.is_break() {
                break;
            }

            if !self.can_continue_operation(gas_per_iteration) {
                return Ok(OperationCompletionStatus::InterruptedBeforeOutOfGas);
            }

            loop_op = process();
        }

        self.clear_operation();

        Ok(OperationCompletionStatus::Completed)
    }

    fn can_continue_operation(&self, operation_cost: u64) -> bool {
        let gas_left = self.blockchain().get_gas_left();

        gas_left > MIN_GAS_TO_SAVE_PROGRESS + operation_cost
    }

    fn save_progress(&self, operation: &OngoingOperationType) {
        self.current_ongoing_operation().set(operation);
    }

    fn clear_operation(&self) {
        self.current_ongoing_operation().clear();
    }

    fn load_select_winners_operation(&self) -> SCResult<(Random<Self::Api>, usize, usize)> {
        let ongoing_operation = self.current_ongoing_operation().get();

        match ongoing_operation {
            OngoingOperationType::None => Ok((
                Random::from_seeds(
                    self.raw_vm_api(),
                    self.blockchain().get_prev_block_random_seed(),
                    self.blockchain().get_block_random_seed(),
                ),
                VEC_MAPPER_START_INDEX,
                self.nr_winning_tickets().get(),
            )),
            OngoingOperationType::SelectWinners {
                seed,
                seed_index,
                ticket_position,
                nr_winning_tickets,
            } => Ok((
                Random::from_hash(self.raw_vm_api(), seed, seed_index),
                ticket_position,
                nr_winning_tickets,
            )),
            _ => sc_error!("Another ongoing operation is in progress"),
        }
    }

    fn load_select_new_winners_operation(
        &self,
        new_first_winning_ticket_position: usize,
    ) -> SCResult<(usize, usize)> {
        let ongoing_operation = self.current_ongoing_operation().get();

        match ongoing_operation {
            OngoingOperationType::None => Ok((
                new_first_winning_ticket_position,
                self.nr_winning_tickets().get(),
            )),
            OngoingOperationType::SelectNewWinners {
                ticket_position,
                nr_winning_tickets,
            } => Ok((ticket_position, nr_winning_tickets)),
            _ => sc_error!("Another ongoing operation is in progress"),
        }
    }

    #[storage_mapper("operation")]
    fn current_ongoing_operation(&self) -> SingleValueMapper<OngoingOperationType>;
}
