multiversx_sc::imports!();

use crate::{config::TokenAmountPair, tickets::WINNING_TICKET};

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

    fn claim_launchpad_tokens<
        SendLaunchpadTokensFn: Fn(&Self, &ManagedAddress, &EsdtTokenPayment<Self::Api>),
    >(
        &self,
        send_fn: SendLaunchpadTokensFn,
    ) {
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
        self.send_launchpad_tokens(&caller, nr_redeemable_tickets, send_fn);
    }

    #[view(hasUserClaimedTokens)]
    fn has_user_claimed(&self, address: &ManagedAddress) -> bool {
        self.claim_list().contains(address)
    }

    // flags

    #[storage_mapper("claimedTokens")]
    fn claim_list(&self) -> WhitelistMapper<Self::Api, ManagedAddress>;
}
