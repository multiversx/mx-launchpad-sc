#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use launchpad_common::{launch_stage::Flags, *};

pub mod locked_launchpad_token_send;

#[multiversx_sc::contract]
pub trait LaunchpadLockedTokens:
    launchpad_common::LaunchpadMain
    + launch_stage::LaunchStageModule
    + config::ConfigModule
    + setup::SetupModule
    + tickets::TicketsModule
    + winner_selection::WinnerSelectionModule
    + ongoing_operation::OngoingOperationModule
    + permissions::PermissionsModule
    + blacklist::BlacklistModule
    + token_send::TokenSendModule
    + user_interactions::UserInteractionsModule
    + locked_launchpad_token_send::LockedLaunchpadTokenSend
    + common_events::CommonEventsModule
    + multiversx_sc_modules::pause::PauseModule
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
        confirmation_period_start_round: u64,
        winner_selection_start_round: u64,
        claim_start_round: u64,
        launchpad_tokens_lock_percentage: u32,
        launchpad_tokens_unlock_epoch: u64,
        simple_lock_sc_address: ManagedAddress,
    ) {
        let flags = Flags {
            has_winner_selection_process_started: false,
            were_tickets_filtered: false,
            were_winners_selected: false,
            was_additional_step_completed: true, // we have no additional step in basic launchpad
        };
        self.init_base(
            launchpad_token_id,
            launchpad_tokens_per_winning_ticket,
            ticket_payment_token,
            ticket_price,
            nr_winning_tickets,
            confirmation_period_start_round,
            winner_selection_start_round,
            claim_start_round,
            flags,
        );

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
        self.add_tickets(address_number_pairs);
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(depositLaunchpadTokens)]
    fn deposit_launchpad_tokens_endpoint(&self) {
        let nr_winning_tickets = self.nr_winning_tickets().get();
        self.deposit_launchpad_tokens(nr_winning_tickets);
    }

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens_endpoint(&self) {
        let _ =
            self.claim_refunded_tickets_and_launchpad_tokens(Self::send_locked_launchpad_tokens);
    }

    #[only_owner]
    #[endpoint(claimTicketPayment)]
    fn claim_ticket_payment_endpoint(&self) {
        self.claim_ticket_payment();
    }

    #[endpoint(addUsersToBlacklist)]
    fn add_users_to_blacklist_endpoint(&self, users_list: MultiValueEncoded<ManagedAddress>) {
        self.add_users_to_blacklist(&users_list.to_vec());
    }
}
