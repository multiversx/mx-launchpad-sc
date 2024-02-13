#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use launchpad_common::{launch_stage::Flags, tickets::WINNING_TICKET};

use crate::guranteed_ticket_winners::GuaranteedTicketsSelectionOperation;

pub mod guaranteed_tickets_init;
pub mod guranteed_ticket_winners;
pub mod token_release;

pub type UserTicketsStatus = MultiValue5<usize, usize, usize, usize, usize>;

#[multiversx_sc::contract]
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
    + token_release::TokenReleaseModule
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
    }

    #[only_owner]
    #[endpoint(addTickets)]
    fn add_tickets_endpoint(
        &self,
        address_number_pairs: MultiValueEncoded<MultiValue4<ManagedAddress, usize, usize, bool>>,
    ) {
        self.add_tickets_with_guaranteed_winners(address_number_pairs);
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(depositLaunchpadTokens)]
    fn deposit_launchpad_tokens_endpoint(&self) {
        let base_selection_winning_tickets = self.nr_winning_tickets().get();
        let reserved_tickets = self.total_guaranteed_tickets().get();
        let total_tickets = base_selection_winning_tickets + reserved_tickets;

        self.deposit_launchpad_tokens(total_tickets);
    }

    #[endpoint(addUsersToBlacklist)]
    fn add_users_to_blacklist_endpoint(&self, users_list: MultiValueEncoded<ManagedAddress>) {
        let users_vec = users_list.to_vec();
        self.add_users_to_blacklist(&users_vec);
        self.clear_users_with_guaranteed_ticket_after_blacklist(&users_vec);
    }

    #[endpoint(removeGuaranteedUsersFromBlacklist)]
    fn remove_guaranteed_users_from_blacklist_endpoint(
        &self,
        users_list: MultiValueEncoded<ManagedAddress>,
    ) {
        let users_vec = users_list.to_vec();
        self.remove_users_from_blacklist(users_list);
        self.remove_guaranteed_tickets_from_blacklist(&users_vec);
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
        let caller = self.blockchain().get_caller();
        let user_results_processed = self.claim_list().contains(&caller);
        if !user_results_processed {
            self.compute_launchpad_results(&caller);
        };

        let claimable_tokens = self.compute_claimable_tokens(&caller);
        if claimable_tokens > 0 {
            let launchpad_token_id = self.launchpad_token_id().get();
            self.send()
                .direct_esdt(&caller, &launchpad_token_id, 0, &claimable_tokens);
            self.user_claimed_balance(&caller)
                .update(|balance| *balance += claimable_tokens);
        }
    }

    fn compute_launchpad_results(&self, caller: &ManagedAddress) {
        self.require_claim_period();

        let ticket_range = self.try_get_ticket_range(caller);
        let nr_confirmed_tickets = self.nr_confirmed_tickets(caller).get();
        let mut nr_redeemable_tickets = 0;

        for ticket_id in ticket_range.first_id..=ticket_range.last_id {
            let ticket_status = self.ticket_status(ticket_id).get();
            if ticket_status == WINNING_TICKET {
                self.ticket_status(ticket_id).clear();

                nr_redeemable_tickets += 1;
            }

            self.ticket_pos_to_id(ticket_id).clear();
        }

        self.nr_confirmed_tickets(caller).clear();
        self.ticket_range_for_address(caller).clear();
        self.ticket_batch(ticket_range.first_id).clear();

        if nr_redeemable_tickets > 0 {
            self.nr_winning_tickets()
                .update(|nr_winning_tickets| *nr_winning_tickets -= nr_redeemable_tickets);
        }

        self.claim_list().add(caller);

        let nr_tickets_to_refund = nr_confirmed_tickets - nr_redeemable_tickets;
        self.refund_ticket_payment(caller, nr_tickets_to_refund);

        if nr_redeemable_tickets > 0 {
            let tokens_per_winning_ticket = self.launchpad_tokens_per_winning_ticket().get();
            let launchpad_tokens_amount_won =
                BigUint::from(nr_redeemable_tickets as u32) * tokens_per_winning_ticket;

            self.user_total_claimable_balance(caller)
                .set(launchpad_tokens_amount_won);
        }
    }

    #[only_owner]
    #[endpoint(claimTicketPayment)]
    fn claim_ticket_payment_endpoint(&self) {
        self.claim_ticket_payment();
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(depositVestingTokens)]
    fn deposit_vesting_tokens(&self) {
        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();
        let launchpad_token_id = self.launchpad_token_id().get();
        require!(payment_token == launchpad_token_id, "Wrong token");
    }

    #[view(getUserTicketsStatus)]
    fn user_tickets_status(&self, address: ManagedAddress) -> UserTicketsStatus {
        let user_ticket_status_mapper = self.user_ticket_status(&address);
        require!(!user_ticket_status_mapper.is_empty(), "User not found");
        let user_ticket_status = user_ticket_status_mapper.get();
        let user_confirmed_tickets_no = self.nr_confirmed_tickets(&address).get();

        (
            user_ticket_status.staking_tickets_allowance,
            user_ticket_status.energy_tickets_allowance,
            user_confirmed_tickets_no,
            user_ticket_status.staking_guaranteed_tickets,
            user_ticket_status.migration_guaranteed_tickets,
        )
            .into()
    }
}
