elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait SetupModule {
    #[allow(clippy::too_many_arguments)]
    #[init]
    fn init(
        &self,
        launchpad_token_id: TokenIdentifier,
        launchpad_tokens_per_winning_ticket: Self::BigUint,
        ticket_payment_token: TokenIdentifier,
        ticket_price: Self::BigUint,
        nr_winning_tickets: usize,
        confirmation_period_start_epoch: u64,
        winner_selection_start_epoch: u64,
        claim_start_epoch: u64,
    ) -> SCResult<()> {
        require!(
            launchpad_token_id.is_valid_esdt_identifier(),
            "Invalid Launchpad token ID"
        );
        self.launchpad_token_id().set(&launchpad_token_id);

        self.try_set_launchpad_tokens_per_winning_ticket(&launchpad_tokens_per_winning_ticket)?;
        self.try_set_ticket_payment_token(&ticket_payment_token)?;
        self.try_set_ticket_price(&ticket_price)?;
        self.try_set_nr_winning_tickets(nr_winning_tickets)?;
        self.try_set_confirmation_period_start_epoch(confirmation_period_start_epoch)?;
        self.try_set_winner_selection_start_epoch(winner_selection_start_epoch)?;
        self.try_set_claim_start_epoch(claim_start_epoch)?;

        self.require_valid_time_periods(
            Some(confirmation_period_start_epoch),
            Some(winner_selection_start_epoch),
            Some(claim_start_epoch),
        )?;

        Ok(())
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(depositLaunchpadTokens)]
    fn deposit_launchpad_tokens(&self) -> SCResult<()> {
        self.require_launchpad_tokens_deposited()
    }

    #[only_owner]
    #[endpoint(setConfirmationPeriodStartEpoch)]
    fn set_confirmation_period_start_epoch(&self, start_epoch: u64) -> SCResult<()> {
        self.require_valid_time_periods(Some(start_epoch), None, None)?;

        self.try_set_confirmation_period_start_epoch(start_epoch)
    }

    #[only_owner]
    #[endpoint(setWinnerSelectionStartEpoch)]
    fn set_winner_selection_start_epoch(&self, start_epoch: u64) -> SCResult<()> {
        self.require_valid_time_periods(None, Some(start_epoch), None)?;

        self.try_set_winner_selection_start_epoch(start_epoch)
    }

    #[only_owner]
    #[endpoint(setClaimStartEpoch)]
    fn set_claim_start_epoch(&self, claim_start_epoch: u64) -> SCResult<()> {
        self.require_valid_time_periods(None, None, Some(claim_start_epoch))?;

        self.try_set_claim_start_epoch(claim_start_epoch)
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

    fn try_set_launchpad_tokens_per_winning_ticket(&self, amount: &Self::BigUint) -> SCResult<()> {
        require!(
            amount > &0,
            "Launchpad tokens per winning ticket cannot be set to zero"
        );

        self.launchpad_tokens_per_winning_ticket().set(amount);

        Ok(())
    }

    fn try_set_nr_winning_tickets(&self, nr_winning_tickets: usize) -> SCResult<()> {
        require!(
            nr_winning_tickets > 0,
            "Cannot set number of winning tickets to zero"
        );

        self.nr_winning_tickets().set(&nr_winning_tickets);

        Ok(())
    }

    fn try_set_confirmation_period_start_epoch(&self, start_epoch: u64) -> SCResult<()> {
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            start_epoch > current_epoch,
            "Confirmation period start epoch cannot be in the past"
        );

        self.confirmation_period_start_epoch().set(&start_epoch);

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

    fn try_set_claim_start_epoch(&self, claim_start_epoch: u64) -> SCResult<()> {
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            claim_start_epoch > current_epoch,
            "Claim start epoch cannot be in the past"
        );

        self.claim_start_epoch().set(&claim_start_epoch);

        Ok(())
    }

    fn require_valid_time_periods(
        &self,
        opt_confirm_start_epoch: Option<u64>,
        opt_winner_selection_start_epoch: Option<u64>,
        opt_claim_start: Option<u64>,
    ) -> SCResult<()> {
        let confirm_start_epoch =
            opt_confirm_start_epoch.unwrap_or_else(|| self.confirmation_period_start_epoch().get());
        let winner_selection_start_epoch = opt_winner_selection_start_epoch
            .unwrap_or_else(|| self.winner_selection_start_epoch().get());
        let claim_start = opt_claim_start.unwrap_or_else(|| self.claim_start_epoch().get());

        require!(
            confirm_start_epoch < winner_selection_start_epoch,
            "Winner selection start epoch must be after confirm start epoch"
        );
        require!(
            winner_selection_start_epoch < claim_start,
            "Claim period must be after winner selection"
        );

        Ok(())
    }

    fn require_launchpad_tokens_deposited(&self) -> SCResult<()> {
        let amount_per_ticket = self.launchpad_tokens_per_winning_ticket().get();
        let total_winning_tickets = self.nr_winning_tickets().get();
        let amount_needed = amount_per_ticket * Self::BigUint::from(total_winning_tickets);

        let launchpad_token_id = self.launchpad_token_id().get();
        let sc_balance = self.blockchain().get_sc_balance(&launchpad_token_id, 0);
        require!(
            sc_balance >= amount_needed,
            "Wrong launchpad tokens amount deposit by owner or not deposited yet"
        );

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

    #[view(getNumberOfWinningTickets)]
    #[storage_mapper("nrWinningTickets")]
    fn nr_winning_tickets(&self) -> SingleValueMapper<Self::Storage, usize>;

    #[view(getConfirmationPeriodStartEpoch)]
    #[storage_mapper("confirmationPeriodStartEpoch")]
    fn confirmation_period_start_epoch(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[view(getWinnerSelectionStart)]
    #[storage_mapper("winnerSelectionStartEpoch")]
    fn winner_selection_start_epoch(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[view(getClaimStartEpoch)]
    #[storage_mapper("claimStartEpoch")]
    fn claim_start_epoch(&self) -> SingleValueMapper<Self::Storage, u64>;
}
