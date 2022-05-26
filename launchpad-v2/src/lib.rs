#![no_std]
#![feature(trait_alias)]

use launchpad_common::launch_stage::Flags;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod claim_nft;
mod confirm_nft;
mod mystery_sft;
mod nft_winners_selection;

#[elrond_wasm::contract]
pub trait Launchpad:
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
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + mystery_sft::MysterySftModule
    + confirm_nft::ConfirmNftModule
    + nft_winners_selection::NftWinnersSelectionModule
    + claim_nft::ClaimNftModule
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
        total_available_nfts: usize,
    ) {
        self.require_valid_cost(&nft_cost);
        require!(total_available_nfts > 0, "Invalid total_available_nfts");

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

        self.nft_cost().set(&nft_cost);
        self.total_available_nfts().set(total_available_nfts);
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
