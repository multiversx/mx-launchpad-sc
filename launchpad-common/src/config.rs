multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct TokenAmountPair<M: ManagedTypeApi> {
    pub token_id: EgldOrEsdtTokenIdentifier<M>,
    pub amount: BigUint<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct TimelineConfig {
    pub confirmation_period_start_round: u64,
    pub winner_selection_start_round: u64,
    pub claim_start_round: u64,
}

#[multiversx_sc::module]
pub trait ConfigModule {
    #[inline]
    fn were_launchpad_tokens_deposited(&self) -> bool {
        self.launchpad_tokens_deposited().get()
    }

    #[view(getConfiguration)]
    #[storage_mapper("configuration")]
    fn configuration(&self) -> SingleValueMapper<TimelineConfig>;

    #[view(getLaunchpadTokenId)]
    #[storage_mapper("launchpadTokenId")]
    fn launchpad_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getLaunchpadTokensPerWinningTicket)]
    #[storage_mapper("launchpadTokensPerWinningTicket")]
    fn launchpad_tokens_per_winning_ticket(&self) -> SingleValueMapper<BigUint>;

    #[view(getTicketPrice)]
    #[storage_mapper("ticketPrice")]
    fn ticket_price(&self) -> SingleValueMapper<TokenAmountPair<Self::Api>>;

    #[view(getNumberOfWinningTickets)]
    #[storage_mapper("nrWinningTickets")]
    fn nr_winning_tickets(&self) -> SingleValueMapper<usize>;

    #[view(getTotalLaunchpadTokensDeposited)]
    #[storage_mapper("totalLaunchpadTokensDeposited")]
    fn total_launchpad_tokens_deposited(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("launchpadTokensDeposited")]
    fn launchpad_tokens_deposited(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("claimableTicketPayment")]
    fn claimable_ticket_payment(&self) -> SingleValueMapper<BigUint>;
}
