elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const MIN_GAS_TO_SAVE_PROGRESS: u64 = 100_000_000;

#[derive(TopDecode, TopEncode, TypeAbi, PartialEq)]
pub enum OngoingOperationType {
    None,
    SelectWinners {
        seed: H256,
        seed_index: usize,
        ticket_position: usize,
    },
    RestartConfirmationPeriod {
        ticket_position: usize,
    },
}

pub enum GasOp {
    Load(OngoingOperationType),
    Continue,
    Save,
    Completed,
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
pub trait OngoingOperationModule {
    fn run_while_it_has_gas<Process>(
        &self,
        initial_op: OngoingOperationType,
        mut process: Process,
    ) -> SCResult<OperationCompletionStatus>
    where
        Process: FnMut(GasOp) -> LoopOp,
    {
        match initial_op {
            OngoingOperationType::None => (),
            other => {
                if process(GasOp::Load(other)).is_break() {
                    return sc_error!("Another ongoing operation is in progress");
                }
            }
        };

        let gas_before = self.blockchain().get_gas_left();

        let mut loop_op = process(GasOp::Continue);

        let gas_after = self.blockchain().get_gas_left();
        let gas_per_iteration = gas_before - gas_after;

        loop {
            if loop_op.is_break() {
                break;
            }

            if !self.can_continue_operation(gas_per_iteration) {
                match process(GasOp::Save) {
                    LoopOp::Save(operation) => self.save_progress(&operation),
                    _ => (),
                }

                return Ok(OperationCompletionStatus::InterruptedBeforeOutOfGas);
            }

            loop_op = process(GasOp::Continue);
        }

        self.clear_operation();
        process(GasOp::Completed);

        Ok(OperationCompletionStatus::Completed)
    }

    fn can_continue_operation(&self, operation_cost: u64) -> bool {
        let gas_left = self.blockchain().get_gas_left();

        gas_left > operation_cost && gas_left - operation_cost > MIN_GAS_TO_SAVE_PROGRESS
    }

    fn save_progress(&self, operation: &OngoingOperationType) {
        self.current_ongoing_operation().set(operation);
    }

    fn clear_operation(&self) {
        self.current_ongoing_operation().clear();
    }

    fn require_no_ongoing_operation(&self) -> SCResult<()> {
        require!(
            matches!(
                self.current_ongoing_operation().get(),
                OngoingOperationType::None
            ),
            "Another ongoing operation is in progress"
        );
        Ok(())
    }

    #[storage_mapper("operation")]
    fn current_ongoing_operation(&self) -> SingleValueMapper<Self::Storage, OngoingOperationType>;
}
