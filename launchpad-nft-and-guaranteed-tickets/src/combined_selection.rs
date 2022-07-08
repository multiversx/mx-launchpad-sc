elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use launchpad_common::random::Random;
use launchpad_guaranteed_tickets::guranteed_ticket_winners::GuaranteedTicketsSelectionOperation;

#[derive(TopDecode, TopEncode)]
pub enum CombinedSelectionStep<M: ManagedTypeApi + CryptoApi> {
    GuaranteedTicketsDistribution {
        op: GuaranteedTicketsSelectionOperation<M>,
    },
    NftSelection {
        rng: Random<M>,
    },
}

impl<M> Default for CombinedSelectionStep<M>
where
    M: ManagedTypeApi + CryptoApi,
{
    fn default() -> Self {
        Self::GuaranteedTicketsDistribution {
            op: GuaranteedTicketsSelectionOperation::default(),
        }
    }
}

#[elrond_wasm::module]
pub trait CombinedSelectionModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::ongoing_operation::OngoingOperationModule
    + launchpad_common::tickets::TicketsModule
    + launchpad_common::permissions::PermissionsModule
    + launchpad_common::user_interactions::UserInteractionsModule
    + launchpad_common::blacklist::BlacklistModule
    + launchpad_common::token_send::TokenSendModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + launchpad_guaranteed_tickets::guaranteed_tickets_init::GuaranteedTicketsInitModule
    + launchpad_guaranteed_tickets::guranteed_ticket_winners::GuaranteedTicketWinnersModule
    + launchpad_with_nft::nft_winners_selection::NftWinnersSelectionModule
    + launchpad_with_nft::confirm_nft::ConfirmNftModule
    + launchpad_with_nft::mystery_sft::MysterySftModule
{
    #[endpoint(secondarySelectionStep)]
    fn secondary_selection_step(&self) -> OperationCompletionStatus {
        self.require_winner_selection_period();

        let flags_mapper = self.flags();
        let mut flags = flags_mapper.get();
        require!(
            flags.were_winners_selected,
            "Must select winners for base launchpad first"
        );
        require!(
            !flags.was_additional_step_completed,
            "Already performed this step"
        );

        let mut current_operation: CombinedSelectionStep<Self::Api> =
            self.load_additional_selection_operation();

        let mut opt_first_op_run_result = None;
        if let CombinedSelectionStep::GuaranteedTicketsDistribution { op } = &mut current_operation
        {
            opt_first_op_run_result = Some(self.select_guaranteed_tickets_substep(op));
        }
        match opt_first_op_run_result {
            Some(OperationCompletionStatus::Completed) => {
                current_operation = CombinedSelectionStep::NftSelection {
                    rng: Random::default(),
                };
            }
            Some(OperationCompletionStatus::InterruptedBeforeOutOfGas) => {
                self.save_additional_selection_progress(&current_operation);

                return OperationCompletionStatus::InterruptedBeforeOutOfGas;
            }
            None => {}
        };

        let mut second_op_run_result = OperationCompletionStatus::Completed;
        if let CombinedSelectionStep::NftSelection { rng } = &mut current_operation {
            second_op_run_result = self.select_nft_winners_substep(rng);
        }

        match second_op_run_result {
            OperationCompletionStatus::Completed => {
                flags.was_additional_step_completed = true;
                flags_mapper.set(&flags);
            }
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                self.save_additional_selection_progress(&current_operation);
            }
        }

        second_op_run_result
    }

    fn select_guaranteed_tickets_substep(
        &self,
        op: &mut GuaranteedTicketsSelectionOperation<Self::Api>,
    ) -> OperationCompletionStatus {
        let first_op_run_result = self.select_guaranteed_tickets(op);
        if first_op_run_result == OperationCompletionStatus::InterruptedBeforeOutOfGas {
            return first_op_run_result;
        }

        let second_op_run_result = self.distribute_leftover_tickets(op);
        if second_op_run_result == OperationCompletionStatus::Completed {
            let ticket_price = self.ticket_price().get();
            let claimable_ticket_payment =
                ticket_price.amount * (op.total_additional_winning_tickets as u32);
            self.claimable_ticket_payment()
                .update(|claim_amt| *claim_amt += claimable_ticket_payment);

            self.nr_winning_tickets()
                .update(|nr_winning| *nr_winning += op.total_additional_winning_tickets);
        }

        second_op_run_result
    }

    fn select_nft_winners_substep(&self, rng: &mut Random<Self::Api>) -> OperationCompletionStatus {
        let op_result = self.select_nft_winners(rng);
        if op_result == OperationCompletionStatus::Completed {
            let winners_selected = self.nft_selection_winners().len();
            let nft_cost = self.nft_cost().get();
            let claimable_nft_payment = nft_cost.amount * winners_selected as u32;
            self.claimable_nft_payment().set(&claimable_nft_payment);
        }

        op_result
    }
}
