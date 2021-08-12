elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait SetupModule {
    #[init]
    fn init(
        &self,
        launchpad_token_id: TokenIdentifier,
        ticket_payment_token: TokenIdentifier,
        ticket_price: Self::BigUint,
        winner_selection_start_epoch: u64,
        confirmation_period_in_epochs: u64,
        claim_period_in_epochs: u64,
    ) -> SCResult<()> {
        require!(
            launchpad_token_id.is_valid_esdt_identifier(),
            "Invalid Launchpad token ID"
        );
        self.launchpad_token_id().set(&launchpad_token_id);

        self.try_set_ticket_payment_token(&ticket_payment_token)?;
        self.try_set_ticket_price(&ticket_price)?;
        self.try_set_winner_selection_start_epoch(winner_selection_start_epoch)?;
        self.try_set_confirmation_period_in_epochs(confirmation_period_in_epochs)?;
        self.try_set_claim_period_in_epochs(claim_period_in_epochs)?;

        Ok(())
    }

    #[only_owner]
    #[endpoint(setWinnerSelectionStartEpoch)]
    fn set_winner_selection_start_epoch(&self, start_epoch: u64) -> SCResult<()> {
        self.try_set_winner_selection_start_epoch(start_epoch)
    }

    #[only_owner]
    #[endpoint]
    fn set_confirmation_period_in_epochs(&self, confirmation_period: u64) -> SCResult<()> {
        self.try_set_confirmation_period_in_epochs(confirmation_period)
    }

    #[only_owner]
    #[endpoint(setClaimPeriodInEpochs)]
    fn set_claim_period_in_epochs(&self, claim_period: u64) -> SCResult<()> {
        self.try_set_claim_period_in_epochs(claim_period)
    }

    #[only_owner]
    #[endpoint(setTicketPaymentToken)]
    fn set_ticket_payment_token(&self, ticket_payment_token: TokenIdentifier) -> SCResult<()> {
        self.try_set_ticket_payment_token(&ticket_payment_token)
    }

    #[only_owner]
    #[endpoint]
    fn set_ticket_price(&self, ticket_price: Self::BigUint) -> SCResult<()> {
        self.try_set_ticket_price(&ticket_price)
    }

    #[only_owner]
    #[endpoint]
    fn set_launchpad_tokens_per_winning_ticket(&self, amount: Self::BigUint) -> SCResult<()> {
        require!(
            amount > 0,
            "Launchpad tokens per winning ticket cannot be set to zero"
        );

        self.launchpad_tokens_per_winning_ticket().set(&amount);

        Ok(())
    }

    #[only_owner]
    #[endpoint(addAddressToBlacklist)]
    fn add_address_to_blacklist(&self, address: Address) -> SCResult<()> {
        self.blacklist().insert(address);

        Ok(())
    }

    #[only_owner]
    #[endpoint(removeAddressFromBlacklist)]
    fn remove_address_from_blacklist(&self, address: Address) -> SCResult<()> {
        self.blacklist().remove(&address);

        Ok(())
    }

    // private

    fn try_set_ticket_payment_token(&self, ticket_payment_token: &TokenIdentifier) -> SCResult<()> {
        require!(
            ticket_payment_token.is_egld() || ticket_payment_token.is_valid_esdt_identifier(),
            "Invalid ticket payment token"
        );

        self.ticket_payment_token().set(ticket_payment_token);

        Ok(())
    }

    fn try_set_ticket_price(&self, ticket_price: &Self::BigUint) -> SCResult<()> {
        require!(ticket_price > &0, "Ticket price must be higher than 0");

        self.ticket_price().set(ticket_price);

        Ok(())
    }

    fn try_set_winner_selection_start_epoch(&self, start_epoch: u64) -> SCResult<()> {
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            start_epoch > current_epoch,
            "Start epoch cannot be in the past"
        );

        self.winner_selection_start_epoch().set(&start_epoch);

        Ok(())
    }

    fn try_set_confirmation_period_in_epochs(&self, confirmation_period: u64) -> SCResult<()> {
        require!(
            confirmation_period > 0,
            "Confirmation period in epochs cannot be set to zero"
        );

        self.confirmation_period_in_epochs().set(&confirmation_period);

        Ok(())
    }

    fn try_set_claim_period_in_epochs(&self, claim_period: u64) -> SCResult<()> {
        require!(
            claim_period > 0,
            "Claim period in epochs cannot be set to zero"
        );

        self.claim_period_in_epochs().set(&claim_period);

        Ok(())
    }

    // storage

    #[view(getLaunchpadTokenId)]
    #[storage_mapper("launchpadTokenId")]
    fn launchpad_token_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(getLaunchpadTokensPerWinningTicket)]
    #[storage_mapper("launchpadTokensPerWinningTicket")]
    fn launchpad_tokens_per_winning_ticket(
        &self,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[view(getTicketPaymentToken)]
    #[storage_mapper("ticketPaymentToken")]
    fn ticket_payment_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(getTicketPrice)]
    #[storage_mapper("ticketPrice")]
    fn ticket_price(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("blacklist")]
    fn blacklist(&self) -> SafeSetMapper<Self::Storage, Address>;

    #[view(getWinnerSelectionStart)]
    #[storage_mapper("winnerSelectionStartEpoch")]
    fn winner_selection_start_epoch(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[view(getConfirmationPeriodInEpochs)]
    #[storage_mapper("confirmationPeriodInEpochs")]
    fn confirmation_period_in_epochs(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[view(getClaimPeriodInEpochs)]
    #[storage_mapper("claimPeriodInEpochs")]
    fn claim_period_in_epochs(&self) -> SingleValueMapper<Self::Storage, u64>;
}
