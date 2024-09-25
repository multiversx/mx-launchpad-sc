multiversx_sc::imports!();

use crate::{common_events, config::TokenAmountPair};

#[multiversx_sc::module]
pub trait TokenSendModule: crate::config::ConfigModule + common_events::CommonEventsModule {
    fn refund_ticket_payment(&self, address: &ManagedAddress, nr_tickets_to_refund: usize) {
        if nr_tickets_to_refund == 0 {
            return;
        }

        let ticket_price: TokenAmountPair<Self::Api> = self.ticket_price().get();
        let ticket_payment_refund_amount = ticket_price.amount * nr_tickets_to_refund as u32;
        self.send().direct(
            address,
            &ticket_price.token_id,
            0,
            &ticket_payment_refund_amount,
        );

        self.emit_refund_ticket_payment_event(
            nr_tickets_to_refund,
            EgldOrEsdtTokenPayment::new(ticket_price.token_id, 0, ticket_payment_refund_amount),
        );
    }

    fn send_launchpad_tokens<
        SendLaunchpadTokensFn: Fn(&Self, &ManagedAddress, &EsdtTokenPayment<Self::Api>),
    >(
        &self,
        address: &ManagedAddress,
        nr_claimed_tickets: usize,
        send_fn: SendLaunchpadTokensFn,
    ) {
        if nr_claimed_tickets == 0 {
            return;
        }

        let launchpad_token_id = self.launchpad_token_id().get();
        let tokens_per_winning_ticket = self.launchpad_tokens_per_winning_ticket().get();
        let launchpad_tokens_amount_to_send =
            BigUint::from(nr_claimed_tickets as u32) * tokens_per_winning_ticket;

        let payment = EsdtTokenPayment::new(launchpad_token_id, 0, launchpad_tokens_amount_to_send);
        send_fn(self, address, &payment);
    }

    fn default_send_launchpad_tokens_fn(
        &self,
        address: &ManagedAddress,
        payment: &EsdtTokenPayment<Self::Api>,
    ) {
        self.send().direct_esdt(
            address,
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
    }
}
