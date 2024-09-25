multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode)]
pub struct RefundTicketPaymentEvent<M: ManagedTypeApi> {
    user: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    tickets_refunded: usize,
    token_payment: EgldOrEsdtTokenPayment<M>,
}

#[derive(TypeAbi, TopEncode)]
pub struct SetTicketPriceEvent<M: ManagedTypeApi> {
    user: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    ticket_price: EgldOrEsdtTokenPayment<M>,
}

#[derive(TypeAbi, TopEncode)]
pub struct ConfirmTicketsEvent<M: ManagedTypeApi> {
    user: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    tickets_confirmed: usize,
    total_confirmed: usize,
    total_tickets: usize,
    token_payment: EgldOrEsdtTokenPayment<M>,
}

#[derive(TypeAbi, TopEncode)]
pub struct FilterTicketsCompletedEvent<M: ManagedTypeApi> {
    user: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    total_tickets_after_filtering: usize,
}

#[derive(TypeAbi, TopEncode)]
pub struct SelectWinnersCompletedEvent<M: ManagedTypeApi> {
    user: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    total_winning_tickets: usize,
}

#[multiversx_sc::module]
pub trait CommonEventsModule {
    fn emit_refund_ticket_payment_event(
        &self,
        tickets_refunded: usize,
        token_payment: EgldOrEsdtTokenPayment<Self::Api>,
    ) {
        let user = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.refund_ticket_payment_event(
            user.clone(),
            block,
            epoch,
            RefundTicketPaymentEvent {
                user,
                block,
                epoch,
                tickets_refunded,
                token_payment,
            },
        )
    }

    fn emit_set_ticket_price_event(&self, ticket_price: EgldOrEsdtTokenPayment<Self::Api>) {
        let user = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.set_ticket_price_event(
            user.clone(),
            block,
            epoch,
            SetTicketPriceEvent {
                user,
                block,
                epoch,
                ticket_price,
            },
        )
    }

    fn emit_confirm_tickets_event(
        &self,
        tickets_confirmed: usize,
        total_confirmed: usize,
        total_tickets: usize,
        token_payment: EgldOrEsdtTokenPayment<Self::Api>,
    ) {
        let user = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.confirm_tickets_event(
            user.clone(),
            block,
            epoch,
            ConfirmTicketsEvent {
                user,
                block,
                epoch,
                tickets_confirmed,
                total_confirmed,
                total_tickets,
                token_payment,
            },
        )
    }

    fn emit_filter_tickets_completed_event(&self, total_tickets_after_filtering: usize) {
        let user = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.filter_tickets_completed_event(
            user.clone(),
            block,
            epoch,
            FilterTicketsCompletedEvent {
                user,
                block,
                epoch,
                total_tickets_after_filtering,
            },
        )
    }

    fn emit_select_winners_completed_event(&self, total_winning_tickets: usize) {
        let user = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.select_winners_completed_event(
            user.clone(),
            block,
            epoch,
            SelectWinnersCompletedEvent {
                user,
                block,
                epoch,
                total_winning_tickets,
            },
        )
    }

    #[event("refundTicketPayment")]
    fn refund_ticket_payment_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        claim_egld_event: RefundTicketPaymentEvent<Self::Api>,
    );

    #[event("setTicketPrice")]
    fn set_ticket_price_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        set_ticket_price_event: SetTicketPriceEvent<Self::Api>,
    );

    #[event("confirmTickets")]
    fn confirm_tickets_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        confirm_tickets_event: ConfirmTicketsEvent<Self::Api>,
    );

    #[event("filterTicketsCompleted")]
    fn filter_tickets_completed_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        filter_tickets_completed_event: FilterTicketsCompletedEvent<Self::Api>,
    );

    #[event("selectWinnersCompleted")]
    fn select_winners_completed_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        select_winners_completed_event: SelectWinnersCompletedEvent<Self::Api>,
    );
}
