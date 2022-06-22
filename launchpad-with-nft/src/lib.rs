#![no_std]
#![feature(trait_alias)]

use launchpad_common::launch_stage::Flags;

use crate::mystery_sft::SftSetupSteps;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod claim_nft;
pub mod confirm_nft;
pub mod mystery_sft;
pub mod nft_blacklist;
pub mod nft_winners_selection;

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
    + nft_blacklist::NftBlacklistModule
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
        ticket_payment_token: EgldOrEsdtTokenIdentifier,
        ticket_price: BigUint,
        nr_winning_tickets: usize,
        confirmation_period_start_epoch: u64,
        winner_selection_start_epoch: u64,
        claim_start_epoch: u64,
        nft_cost: EgldOrEsdtTokenPayment<Self::Api>,
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
        self.sft_setup_steps().set_if_empty(&SftSetupSteps {
            issued_token: false,
            created_initial_tokens: false,
            set_transfer_role: false,
        });
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

    #[endpoint(addUsersToBlacklist)]
    fn add_users_to_blacklist_endpoint(&self, users_list: MultiValueEncoded<ManagedAddress>) {
        let users_list_vec = users_list.to_vec();
        self.add_users_to_blacklist(&users_list_vec);
        self.refund_nft_cost_after_blacklist(&users_list_vec);
    }

    #[view(hasUserConfirmedNft)]
    fn has_user_confirmed_nft(&self, user: ManagedAddress) -> bool {
        self.confirmed_nft_user_list().contains(&user)
            || self.nft_selection_winners().contains(&user)
    }

    #[view(hasUserWonNft)]
    fn has_user_won_nft(&self, user: ManagedAddress) -> bool {
        self.nft_selection_winners().contains(&user)
    }

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens_endpoint(&self) {
        self.claim_launchpad_tokens();
        self.claim_nft();
    }
}
