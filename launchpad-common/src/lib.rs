#![no_std]
#![feature(trait_alias)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod blacklist;
pub mod config;
pub mod launch_stage;
pub mod ongoing_operation;
pub mod permissions;
pub mod random;
pub mod setup;
pub mod tickets;
pub mod token_send;
pub mod user_interactions;
pub mod winner_selection;

use config::{EpochsConfig, TokenAmountPair};
use launch_stage::Flags;
use tickets::FIRST_TICKET_ID;

#[elrond_wasm::module]
pub trait LaunchpadMain:
    launch_stage::LaunchStageModule
    + config::ConfigModule
    + setup::SetupModule
    + tickets::TicketsModule
    + winner_selection::WinnerSelectionModule
    + ongoing_operation::OngoingOperationModule
    + permissions::PermissionsModule
    + blacklist::BlacklistModule
    + token_send::TokenSendModule
    + user_interactions::UserInteractionsModule
{
    #[allow(clippy::too_many_arguments)]
    fn init_base(
        &self,
        launchpad_token_id: TokenIdentifier,
        launchpad_tokens_per_winning_ticket: BigUint,
        ticket_payment_token: EgldOrEsdtTokenIdentifier,
        ticket_price: BigUint,
        nr_winning_tickets: usize,
        confirmation_period_start_epoch: u64,
        winner_selection_start_epoch: u64,
        claim_start_epoch: u64,
        flags: Flags,
    ) {
        self.launchpad_token_id().set(&launchpad_token_id);

        self.try_set_launchpad_tokens_per_winning_ticket(&launchpad_tokens_per_winning_ticket);
        self.try_set_ticket_price(ticket_payment_token, ticket_price);
        self.try_set_nr_winning_tickets(nr_winning_tickets);

        let config = EpochsConfig {
            confirmation_period_start_epoch,
            winner_selection_start_epoch,
            claim_start_epoch,
        };
        self.require_valid_time_periods(&config);
        self.configuration().set(&config);
        self.flags().set_if_empty(&flags);

        let caller = self.blockchain().get_caller();
        self.support_address().set_if_empty(&caller);
    }

    #[only_owner]
    #[endpoint(claimTicketPayment)]
    fn claim_ticket_payment(&self) {
        self.require_claim_period();

        let owner = self.blockchain().get_caller();

        let ticket_payment_mapper = self.claimable_ticket_payment();
        let claimable_ticket_payment = ticket_payment_mapper.get();
        if claimable_ticket_payment > 0 {
            ticket_payment_mapper.clear();

            let ticket_price: TokenAmountPair<Self::Api> = self.ticket_price().get();
            self.send().direct(
                &owner,
                &ticket_price.token_id,
                0,
                &claimable_ticket_payment,
                &[],
            );
        }

        let launchpad_token_id = self.launchpad_token_id().get();
        let launchpad_tokens_needed = self.get_exact_launchpad_tokens_needed();
        let launchpad_tokens_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &launchpad_token_id,
            0,
        );
        let extra_launchpad_tokens = launchpad_tokens_balance - launchpad_tokens_needed;
        if extra_launchpad_tokens > 0 {
            self.send()
                .direct_esdt(&owner, &launchpad_token_id, 0, &extra_launchpad_tokens, &[]);
        }
    }
}
