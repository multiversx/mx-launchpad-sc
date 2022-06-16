#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use launchpad_common::{launch_stage::Flags, *};

pub mod guranteed_ticket_winners;

#[elrond_wasm::contract]
pub trait LaunchpadGuaranteedTickets:
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
        self.require_add_tickets_period();

        let max_tier_tickets = self.max_tier_tickets().get();
        let mut max_tier_whitelist = self.max_tier_users();
        let mut total_winning_tickets = self.nr_winning_tickets().get();

        for multi_arg in address_number_pairs {
            let (buyer, nr_tickets) = multi_arg.into_tuple();
            require!(nr_tickets < max_tier_tickets, "Too many tickets for user");

            self.try_create_tickets(buyer.clone(), nr_tickets);

            if nr_tickets == max_tier_tickets {
                require!(total_winning_tickets > 0, "Too many max tier users");

                let _ = max_tier_whitelist.insert(buyer);
                total_winning_tickets -= 1;
            }
        }

        self.nr_winning_tickets().set(total_winning_tickets);
    }

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens_endpoint(&self) {
        self.claim_launchpad_tokens();
    }

    #[endpoint(addUsersToBlacklist)]
    fn add_users_to_blacklist_endpoint(&self, users_list: MultiValueEncoded<ManagedAddress>) {
        let users_vec = users_list.to_vec();
        self.add_users_to_blacklist(&users_vec);

        let mut max_tier_whitelist = self.max_tier_users();
        for user in &users_vec {
            let _ = max_tier_whitelist.swap_remove(&user);
        }
    }
}
