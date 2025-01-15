multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{
    config::TokenAmountPair,
    tickets::{TicketRange, WINNING_TICKET},
};

#[derive(TypeAbi, TopEncode, TopDecode)]
pub enum ClaimType {
    None,
    RefundedTickets,
    All,
}

pub struct ClaimRefundedTicketsResultType<M: ManagedTypeApi> {
    pub winning_ticket_ids: ManagedVec<M, usize>,
}

#[multiversx_sc::module]
pub trait UserInteractionsModule:
    crate::launch_stage::LaunchStageModule
    + crate::config::ConfigModule
    + crate::blacklist::BlacklistModule
    + crate::tickets::TicketsModule
    + crate::token_send::TokenSendModule
    + crate::permissions::PermissionsModule
    + crate::common_events::CommonEventsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[payable("*")]
    #[endpoint(confirmTickets)]
    fn confirm_tickets(&self, nr_tickets_to_confirm: usize) {
        self.require_not_paused();
        let (payment_token, payment_amount) = self.call_value().egld_or_single_fungible_esdt();

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

        self.nr_confirmed_tickets(&caller).set(total_confirmed);

        let token_payment = EgldOrEsdtTokenPayment::new(payment_token, 0, payment_amount);
        self.emit_confirm_tickets_event(
            nr_tickets_to_confirm,
            total_confirmed,
            total_tickets,
            token_payment,
        );
    }

    fn claim_refunded_tickets_and_launchpad_tokens<
        SendLaunchpadTokensFn: Fn(&Self, &ManagedAddress, &EsdtTokenPayment),
    >(
        &self,
        send_fn: SendLaunchpadTokensFn,
    ) {
        let winning_ticket_ids = self.claim_refunded_tickets().winning_ticket_ids;
        self.claim_launchpad_tokens(winning_ticket_ids, send_fn);
    }

    fn claim_refunded_tickets(&self) -> ClaimRefundedTicketsResultType<Self::Api> {
        let flags = self.flags().get();
        require!(
            flags.were_winners_selected && flags.was_additional_step_completed,
            "Not in claim period"
        );

        let caller = self.blockchain().get_caller();
        let ticket_range = self.try_get_ticket_range(&caller);
        let winning_ticket_ids = self.get_winning_ticket_ids(&ticket_range);

        let claim_status_mapper = self.claimed_tokens(&caller);
        let claim_status = claim_status_mapper.get();
        match claim_status {
            ClaimType::None => {}
            ClaimType::RefundedTickets => {
                return ClaimRefundedTicketsResultType { winning_ticket_ids }
            }
            ClaimType::All => sc_panic!("Already claimed"),
        };

        let nr_redeemable_tickets = winning_ticket_ids.len();
        if nr_redeemable_tickets > 0 {
            self.nr_winning_tickets()
                .update(|nr_winning_tickets| *nr_winning_tickets -= nr_redeemable_tickets);
        }

        claim_status_mapper.set(ClaimType::RefundedTickets);

        let nr_confirmed_tickets = self.nr_confirmed_tickets(&caller).get();
        let nr_tickets_to_refund = nr_confirmed_tickets - nr_redeemable_tickets;
        self.refund_ticket_payment(&caller, nr_tickets_to_refund);

        ClaimRefundedTicketsResultType { winning_ticket_ids }
    }

    fn claim_launchpad_tokens<
        SendLaunchpadTokensFn: Fn(&Self, &ManagedAddress, &EsdtTokenPayment),
    >(
        &self,
        winning_ticket_ids: ManagedVec<usize>,
        send_fn: SendLaunchpadTokensFn,
    ) {
        self.require_claim_period();

        let caller = self.blockchain().get_caller();
        let ticket_range = self.try_get_ticket_range(&caller);

        for ticket_id in &winning_ticket_ids {
            self.ticket_status(ticket_id).clear();
        }

        self.nr_confirmed_tickets(&caller).clear();
        self.ticket_range_for_address(&caller).clear();
        self.ticket_batch(ticket_range.first_id).clear();

        self.claimed_tokens(&caller).set(ClaimType::All);

        let nr_redeemable_tickets = winning_ticket_ids.len();
        self.send_launchpad_tokens(&caller, nr_redeemable_tickets, send_fn);
    }

    fn get_winning_ticket_ids(&self, ticket_range: &TicketRange) -> ManagedVec<usize> {
        let mut winning_ticket_ids = ManagedVec::new();
        for ticket_id in ticket_range.first_id..=ticket_range.last_id {
            let ticket_status = self.ticket_status(ticket_id).get();
            if ticket_status == WINNING_TICKET {
                winning_ticket_ids.push(ticket_id);
            }

            self.ticket_pos_to_id(ticket_id).clear();
        }

        winning_ticket_ids
    }

    #[view(getClaimTypeForUser)]
    #[storage_mapper("claimedTokens")]
    fn claimed_tokens(&self, user: &ManagedAddress) -> SingleValueMapper<ClaimType>;
}
