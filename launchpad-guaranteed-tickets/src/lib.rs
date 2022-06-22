#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use launchpad_common::launch_stage::Flags;

use crate::guranteed_ticket_winners::GuaranteedTicketsSelectionOperation;

pub mod guaranteed_tickets_init;
pub mod guranteed_ticket_winners;

#[elrond_wasm::contract]
pub trait LaunchpadGuaranteedTickets:
    launchpad_common::LaunchpadMain
    + launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::setup::SetupModule
    + launchpad_common::tickets::TicketsModule
    + launchpad_common::winner_selection::WinnerSelectionModule
    + launchpad_common::ongoing_operation::OngoingOperationModule
    + launchpad_common::permissions::PermissionsModule
    + launchpad_common::blacklist::BlacklistModule
    + launchpad_common::token_send::TokenSendModule
    + launchpad_common::user_interactions::UserInteractionsModule
    + guaranteed_tickets_init::GuaranteedTicketsInitModule
    + guranteed_ticket_winners::GuaranteedTicketWinnersModule
{
    #[allow(clippy::too_many_arguments)]
    #[init]
    fn init(
        &self,
        launchpad_token_id: TokenIdentifier,
        launchpad_tokens_per_winning_ticket: BigUint,
        ticket_payment_token: EgldOrEsdtTokenIdentifier,
        ticket_price: BigUint,
        nr_winning_tickets: usize,
        confirmation_period_start_epoch: u64,
        winner_selection_start_epoch: u64,
        claim_start_epoch: u64,
        max_tier_tickets: usize,
    ) {
        self.init_base(
            launchpad_token_id,
            launchpad_tokens_per_winning_ticket,
            ticket_payment_token,
            ticket_price,
            nr_winning_tickets,
            confirmation_period_start_epoch,
            winner_selection_start_epoch,
            claim_start_epoch,
            Flags::default(),
        );

        require!(max_tier_tickets > 0, "Invalid max tier ticket number");
        self.max_tier_tickets().set(max_tier_tickets);
    }

    #[only_owner]
    #[endpoint(addTickets)]
    fn add_tickets_endpoint(
        &self,
        address_number_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) {
        self.add_tickets_with_guaranteed_winners(address_number_pairs);
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(depositLaunchpadTokens)]
    fn deposit_launchpad_tokens_endpoint(&self) {
        let base_selection_winning_tickets = self.nr_winning_tickets().get();
        let reserved_tickets = self.max_tier_users().len();
        let total_tickets = base_selection_winning_tickets + reserved_tickets;

        self.deposit_launchpad_tokens(total_tickets);
    }

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens_endpoint(&self) {
        self.claim_launchpad_tokens();
    }

    #[endpoint(addUsersToBlacklist)]
    fn add_users_to_blacklist_endpoint(&self, users_list: MultiValueEncoded<ManagedAddress>) {
        let users_vec = users_list.to_vec();
        self.add_users_to_blacklist(&users_vec);
        self.clear_max_tier_users_after_blacklist(&users_vec);
    }

    #[endpoint(distributeGuaranteedTickets)]
    fn distribute_guaranteed_tickets_endpoint(&self) -> OperationCompletionStatus {
        self.require_winner_selection_period();

        let flags_mapper = self.flags();
        let mut flags = flags_mapper.get();
        require!(
            flags.were_winners_selected,
            "Must select winners for base launchpad first"
        );
        require!(
            !flags.was_additional_step_completed,
            "Already distributed tickets"
        );

        let mut current_operation: GuaranteedTicketsSelectionOperation<Self::Api> =
            self.load_additional_selection_operation();
        let first_op_run_result = self.select_guaranteed_tickets(&mut current_operation);
        if first_op_run_result == OperationCompletionStatus::InterruptedBeforeOutOfGas {
            self.save_custom_operation(&current_operation);

            return first_op_run_result;
        }

        let second_op_run_result = self.distribute_leftover_tickets(&mut current_operation);
        match second_op_run_result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                self.save_custom_operation(&current_operation);
            }
            OperationCompletionStatus::Completed => {
                flags.was_additional_step_completed = true;
                flags_mapper.set(&flags);

                let ticket_price = self.ticket_price().get();
                let claimable_ticket_payment = ticket_price.amount
                    * (current_operation.total_additional_winning_tickets as u32);
                self.claimable_ticket_payment()
                    .update(|claim_amt| *claim_amt += claimable_ticket_payment);

                self.nr_winning_tickets().update(|nr_winning| {
                    *nr_winning += current_operation.total_additional_winning_tickets
                });
            }
        };

        second_op_run_result
    }
}
