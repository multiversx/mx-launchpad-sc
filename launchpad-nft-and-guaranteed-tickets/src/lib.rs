#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use launchpad_common::launch_stage::Flags;
use launchpad_with_nft::mystery_sft::SftSetupSteps;

pub mod combined_selection;

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
    + launchpad_guaranteed_tickets::guaranteed_tickets_init::GuaranteedTicketsInitModule
    + launchpad_guaranteed_tickets::guranteed_ticket_winners::GuaranteedTicketWinnersModule
    + launchpad_with_nft::nft_blacklist::NftBlacklistModule
    + launchpad_with_nft::mystery_sft::MysterySftModule
    + launchpad_with_nft::confirm_nft::ConfirmNftModule
    + launchpad_with_nft::nft_winners_selection::NftWinnersSelectionModule
    + launchpad_with_nft::claim_nft::ClaimNftModule
    + combined_selection::CombinedSelectionModule
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
        nft_cost_token_id: EgldOrEsdtTokenIdentifier,
        nft_cost_token_nonce: u64,
        nft_cost_token_amount: BigUint,
        total_available_nfts: usize,
        max_tier_tickets: usize,
    ) {
        let nft_cost = EgldOrEsdtTokenPayment::new(
            nft_cost_token_id,
            nft_cost_token_nonce,
            nft_cost_token_amount,
        );

        self.require_valid_cost(&nft_cost);
        require!(total_available_nfts > 0, "Invalid total_available_nfts");

        require!(max_tier_tickets > 0, "Invalid max tier ticket number");
        self.max_tier_tickets().set(max_tier_tickets);

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
        self.sft_setup_steps()
            .set_if_empty(&SftSetupSteps::default());
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

    #[endpoint(addUsersToBlacklist)]
    fn add_users_to_blacklist_endpoint(&self, users_list: MultiValueEncoded<ManagedAddress>) {
        let users_list_vec = users_list.to_vec();
        self.add_users_to_blacklist(&users_list_vec);
        self.clear_max_tier_users_after_blacklist(&users_list_vec);
        self.refund_nft_cost_after_blacklist(&users_list_vec);
    }

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens_endpoint(&self) {
        self.claim_launchpad_tokens();
        self.claim_nft();
    }

    #[only_owner]
    #[endpoint(claimTicketPayment)]
    fn claim_ticket_payment_endpoint(&self) {
        self.claim_ticket_payment();
        self.claim_nft_payment();
    }
}
