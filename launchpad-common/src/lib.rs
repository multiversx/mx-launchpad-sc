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

use config::EpochsConfig;
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
}
