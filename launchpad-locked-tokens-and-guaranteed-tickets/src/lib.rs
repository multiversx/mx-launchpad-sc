#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use launchpad_common::launch_stage::Flags;
use launchpad_guaranteed_tickets::guranteed_ticket_winners::GuaranteedTicketsSelectionOperation;

#[elrond_wasm::contract]
pub trait LaunchpadLockedTokensAndGuaranteedTickets:
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
    + launchpad_guaranteed_tickets::guaranteed_tickets_init::GuaranteedTicketsInitModule
    + launchpad_guaranteed_tickets::guranteed_ticket_winners::GuaranteedTicketWinnersModule
    + launchpad_locked_tokens::locked_launchpad_token_send::LockedLaunchpadTokenSend
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
        confirmation_period_start_block: u64,
        winner_selection_start_block: u64,
        claim_start_block: u64,
        min_confirmed_for_guaranteed_ticket: usize,
        launchpad_tokens_lock_percentage: u32,
        launchpad_tokens_unlock_epoch: u64,
        simple_lock_sc_address: ManagedAddress,
    ) {
        self.init_base(
            launchpad_token_id,
            launchpad_tokens_per_winning_ticket,
            ticket_payment_token,
            ticket_price,
            nr_winning_tickets,
            confirmation_period_start_block,
            winner_selection_start_block,
            claim_start_block,
            Flags::default(),
        );

        require!(
            min_confirmed_for_guaranteed_ticket > 0,
            "Invalid minimum tickets confirmed for guaranteed winning ticket"
        );
        self.min_confirmed_for_guaranteed_ticket()
            .set(min_confirmed_for_guaranteed_ticket);

        self.try_set_launchpad_tokens_lock_percentage(launchpad_tokens_lock_percentage);
        self.try_set_launchpad_tokens_unlock_epoch(launchpad_tokens_unlock_epoch);
        self.try_set_simple_lock_sc_address(simple_lock_sc_address);
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
        let reserved_tickets = self.users_with_guaranteed_ticket().len();
        let total_tickets = base_selection_winning_tickets + reserved_tickets;

        self.deposit_launchpad_tokens(total_tickets);
    }

    #[endpoint(addUsersToBlacklist)]
    fn add_users_to_blacklist_endpoint(&self, users_list: MultiValueEncoded<ManagedAddress>) {
        let users_vec = users_list.to_vec();
        self.add_users_to_blacklist(&users_vec);
        self.clear_users_with_guaranteed_ticket_after_blacklist(&users_vec);
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
            self.save_additional_selection_progress(&current_operation);

            return first_op_run_result;
        }

        let second_op_run_result = self.distribute_leftover_tickets(&mut current_operation);
        match second_op_run_result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                self.save_additional_selection_progress(&current_operation);
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

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens_endpoint(&self) {
        self.claim_launchpad_tokens(Self::send_locked_launchpad_tokens);
    }

    #[only_owner]
    #[endpoint(claimTicketPayment)]
    fn claim_ticket_payment_endpoint(&self) {
        self.claim_ticket_payment();
    }
}
