elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait SetupModule: crate::launch_stage::LaunchStageModule {
    #[only_owner]
    #[payable("*")]
    #[endpoint(depositLaunchpadTokens)]
    fn deposit_launchpad_tokens(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
    ) {
        require!(
            !self.were_launchpad_tokens_deposited(),
            "Tokens already deposited"
        );

        let launchpad_token_id = self.launchpad_token_id().get();
        require!(payment_token == launchpad_token_id, "Wrong token");

        let amount_needed = self.get_exact_lanchpad_tokens_needed();
        require!(payment_amount == amount_needed, "Wrong amount");

        self.launchpad_tokens_deposited().set(&true);
    }

    #[only_owner]
    #[endpoint(setTicketPaymentToken)]
    fn set_ticket_payment_token(&self, ticket_payment_token: TokenIdentifier) {
        self.require_add_tickets_period();
        self.ticket_payment_token().set(&ticket_payment_token);
    }

    #[only_owner]
    #[endpoint(setTicketPrice)]
    fn set_ticket_price(&self, ticket_price: BigUint) {
        self.require_add_tickets_period();
        self.try_set_ticket_price(&ticket_price)
    }

    #[only_owner]
    #[endpoint(setLaunchpadTokensPerWinningTicket)]
    fn set_launchpad_tokens_per_winning_ticket(&self, amount: BigUint) {
        self.require_add_tickets_period();
        self.try_set_launchpad_tokens_per_winning_ticket(&amount)
    }

    #[only_owner]
    #[endpoint(setConfirmationPeriodStartEpoch)]
    fn set_confirmation_period_start_epoch(&self, start_epoch: u64) {
        //let old_start_epoch = self.confirmation_period_start_epoch().get();
        //self.require_valid_config_epoch_change(old_start_epoch);
        //self.require_valid_time_periods(Some(start_epoch), None, None);

        self.try_set_confirmation_period_start_epoch(start_epoch)
    }

    #[only_owner]
    #[endpoint(setWinnerSelectionStartEpoch)]
    fn set_winner_selection_start_epoch(&self, start_epoch: u64) {
        //let old_start_epoch = self.winner_selection_start_epoch().get();

        //self.require_valid_config_epoch_change(old_start_epoch);
        //self.require_valid_time_periods(None, Some(start_epoch), None);

        self.try_set_winner_selection_start_epoch(start_epoch)
    }

    #[only_owner]
    #[endpoint(setClaimStartEpoch)]
    fn set_claim_start_epoch(&self, claim_start_epoch: u64) {
        //let old_start_epoch = self.claim_start_epoch().get();
        
        //self.require_valid_config_epoch_change(old_start_epoch);
        //self.require_valid_time_periods(None, None, Some(claim_start_epoch));

        self.try_set_claim_start_epoch(claim_start_epoch)
    }

    // private

    fn get_exact_lanchpad_tokens_needed(&self) -> BigUint {
        let amount_per_ticket = self.launchpad_tokens_per_winning_ticket().get();
        let total_winning_tickets = self.nr_winning_tickets().get();

        amount_per_ticket * (total_winning_tickets as u32)
    }

    fn try_set_ticket_price(&self, ticket_price: &BigUint) {
        require!(ticket_price > &0, "Ticket price must be higher than 0");

        self.ticket_price().set(ticket_price);
    }

    fn try_set_launchpad_tokens_per_winning_ticket(&self, amount: &BigUint) {
        require!(
            amount > &0,
            "Launchpad tokens per winning ticket cannot be set to zero"
        );

        self.launchpad_tokens_per_winning_ticket().set(amount);
    }

    fn try_set_nr_winning_tickets(&self, nr_winning_tickets: usize) {
        require!(
            nr_winning_tickets > 0,
            "Cannot set number of winning tickets to zero"
        );

        self.nr_winning_tickets().set(&nr_winning_tickets);
    }

    fn try_set_confirmation_period_start_epoch(&self, start_epoch: u64) {
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            start_epoch > current_epoch,
            "Confirmation period start epoch cannot be in the past"
        );

        self.confirmation_period_start_epoch().set(&start_epoch);
    }

    fn try_set_winner_selection_start_epoch(&self, start_epoch: u64) {
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            start_epoch > current_epoch,
            "Start epoch cannot be in the past"
        );

        self.winner_selection_start_epoch().set(&start_epoch);
    }

    fn try_set_claim_start_epoch(&self, claim_start_epoch: u64) {
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            claim_start_epoch > current_epoch,
            "Claim start epoch cannot be in the past"
        );

        self.claim_start_epoch().set(&claim_start_epoch);
    }

    fn require_valid_config_epoch_change(&self, old_start_epoch: u64) {
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            old_start_epoch > current_epoch,
            "Cannot change start epoch, it's either in progress or passed already"
        );
    }

    fn require_valid_time_periods(
        &self,
        opt_confirm_start_epoch: Option<u64>,
        opt_winner_selection_start_epoch: Option<u64>,
        opt_claim_start: Option<u64>,
    ) {
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
            winner_selection_start_epoch <= claim_start,
            "Claim period must be after winner selection"
        );
    }

    #[inline(always)]
    fn were_launchpad_tokens_deposited(&self) -> bool {
        self.launchpad_tokens_deposited().get()
    }

    // storage

    #[view(getLaunchpadTokenId)]
    #[storage_mapper("launchpadTokenId")]
    fn launchpad_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getLaunchpadTokensPerWinningTicket)]
    #[storage_mapper("launchpadTokensPerWinningTicket")]
    fn launchpad_tokens_per_winning_ticket(&self) -> SingleValueMapper<BigUint>;

    #[view(getTicketPaymentToken)]
    #[storage_mapper("ticketPaymentToken")]
    fn ticket_payment_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getTicketPrice)]
    #[storage_mapper("ticketPrice")]
    fn ticket_price(&self) -> SingleValueMapper<BigUint>;

    #[view(getNumberOfWinningTickets)]
    #[storage_mapper("nrWinningTickets")]
    fn nr_winning_tickets(&self) -> SingleValueMapper<usize>;

    // flags

    #[storage_mapper("launchpadTokensDeposited")]
    fn launchpad_tokens_deposited(&self) -> SingleValueMapper<bool>;
}
