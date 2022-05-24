#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod blacklist;
mod config;
mod launch_stage;
mod ongoing_operation;
mod permissions;
mod random;
mod setup;
mod tickets;
mod token_send;
mod winner_selection;

use config::{EpochsConfig, TokenAmountPair};
use launch_stage::Flags;
use tickets::{FIRST_TICKET_ID, WINNING_TICKET};

#[elrond_wasm::contract]
pub trait Launchpad:
    launch_stage::LaunchStageModule
    + config::ConfigModule
    + setup::SetupModule
    + tickets::TicketsModule
    + winner_selection::WinnerSelectionModule
    + ongoing_operation::OngoingOperationModule
    + permissions::PermissionsModule
    + blacklist::BlacklistModule
    + token_send::TokenSendModule
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
        self.flags().set_if_empty(&Flags {
            were_tickets_filtered: false,
            were_winners_selected: false,
            has_winner_selection_process_started: false,
        });

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
        let launchpad_tokens_balance = self.blockchain().get_sc_balance(&launchpad_token_id, 0);
        let extra_launchpad_tokens = launchpad_tokens_balance - launchpad_tokens_needed;
        if extra_launchpad_tokens > 0 {
            self.send()
                .direct(&owner, &launchpad_token_id, 0, &extra_launchpad_tokens, &[]);
        }
    }

    #[payable("*")]
    #[endpoint(confirmTickets)]
    fn confirm_tickets(&self, nr_tickets_to_confirm: usize) {
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();

        self.require_confirmation_period();
        require!(
            self.were_launchpad_tokens_deposited(),
            "Launchpad tokens not deposited yet"
        );

        let caller = self.blockchain().get_caller();
        require!(
            !self.is_user_blacklisted(&caller),
            "You have been put into the blacklist and may not confirm tickets"
        );

        let total_tickets = self.get_total_number_of_tickets_for_address(&caller);
        let nr_confirmed = self.nr_confirmed_tickets(&caller).get();
        let total_confirmed = nr_confirmed + nr_tickets_to_confirm;
        require!(
            total_confirmed <= total_tickets,
            "Trying to confirm too many tickets"
        );

        let ticket_price: TokenAmountPair<Self::Api> = self.ticket_price().get();
        let total_ticket_price = ticket_price.amount * nr_tickets_to_confirm as u32;
        require!(
            payment_token == ticket_price.token_id,
            "Wrong payment token used"
        );
        require!(payment_amount == total_ticket_price, "Wrong amount sent");

        self.nr_confirmed_tickets(&caller).set(&total_confirmed);
    }

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens(&self) {
        self.require_claim_period();

        let caller = self.blockchain().get_caller();
        require!(!self.has_user_claimed(&caller), "Already claimed");

        let ticket_range = self.try_get_ticket_range(&caller);
        let nr_confirmed_tickets = self.nr_confirmed_tickets(&caller).get();
        let mut nr_redeemable_tickets = 0;

        for ticket_id in ticket_range.first_id..=ticket_range.last_id {
            let ticket_status = self.ticket_status(ticket_id).get();
            if ticket_status == WINNING_TICKET {
                self.ticket_status(ticket_id).clear();

                nr_redeemable_tickets += 1;
            }

            self.ticket_pos_to_id(ticket_id).clear();
        }

        self.nr_confirmed_tickets(&caller).clear();
        self.ticket_range_for_address(&caller).clear();
        self.ticket_batch(ticket_range.first_id).clear();

        if nr_redeemable_tickets > 0 {
            self.nr_winning_tickets()
                .update(|nr_winning_tickets| *nr_winning_tickets -= nr_redeemable_tickets);
        }

        self.claim_list().add(&caller);

        let nr_tickets_to_refund = nr_confirmed_tickets - nr_redeemable_tickets;
        self.refund_ticket_payment(&caller, nr_tickets_to_refund);
        self.send_launchpad_tokens(&caller, nr_redeemable_tickets);
    }

    #[view(hasUserClaimedTokens)]
    fn has_user_claimed(&self, address: &ManagedAddress) -> bool {
        self.claim_list().contains(address)
    }

    // flags

    #[storage_mapper("claimedTokens")]
    fn claim_list(&self) -> WhitelistMapper<Self::Api, ManagedAddress>;
}
