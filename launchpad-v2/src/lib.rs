#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod confirm_nft;
mod mystery_sft;

use launchpad_common::{launch_stage::Flags, *};

#[elrond_wasm::contract]
pub trait Launchpad:
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
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + mystery_sft::MysterySftModule
    + confirm_nft::ConfirmNftModule
{
    #[allow(clippy::too_many_arguments)]
    #[init]
    fn init(
        &self,
        launchpad_token_id: TokenIdentifier,
        launchpad_tokens_per_winning_ticket: BigUint,
        ticket_payment_token: TokenIdentifier,
        ticket_price: BigUint,
        nr_winning_tickets: usize,
        confirmation_period_start_epoch: u64,
        winner_selection_start_epoch: u64,
        claim_start_epoch: u64,
        nft_cost: EsdtTokenPayment<Self::Api>,
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

        self.require_valid_cost(&nft_cost);
        self.nft_cost().set(&nft_cost);
    }

    fn require_valid_cost(&self, cost: &EsdtTokenPayment<Self::Api>) {
        if cost.token_identifier.is_egld() {
            require!(cost.token_nonce == 0, "EGLD token has no nonce");
        } else {
            require!(
                cost.token_identifier.is_valid_esdt_identifier(),
                "Invalid ESDT token ID"
            );
        }

        require!(cost.amount > 0, "Cost may not be 0");
    }
}
