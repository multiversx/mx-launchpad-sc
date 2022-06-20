elrond_wasm::imports!();

use crate::config::{EpochsConfig, TokenAmountPair};

#[elrond_wasm::module]
pub trait SetupModule:
    crate::launch_stage::LaunchStageModule + crate::config::ConfigModule
{
    fn deposit_launchpad_tokens(&self, total_winning_tickets: usize) {
        require!(
            !self.were_launchpad_tokens_deposited(),
            "Tokens already deposited"
        );

        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();
        let launchpad_token_id = self.launchpad_token_id().get();
        require!(payment_token == launchpad_token_id, "Wrong token");

        let amount_per_ticket = self.launchpad_tokens_per_winning_ticket().get();
        let amount_needed = amount_per_ticket * (total_winning_tickets as u32);
        require!(payment_amount == amount_needed, "Wrong amount");

        self.launchpad_tokens_deposited().set(&true);
    }

    #[only_owner]
    #[endpoint(setTicketPrice)]
    fn set_ticket_price(&self, token_id: EgldOrEsdtTokenIdentifier, amount: BigUint) {
        self.require_add_tickets_period();
        self.try_set_ticket_price(token_id, amount);
    }

    #[only_owner]
    #[endpoint(setLaunchpadTokensPerWinningTicket)]
    fn set_launchpad_tokens_per_winning_ticket(&self, amount: BigUint) {
        self.require_add_tickets_period();
        self.try_set_launchpad_tokens_per_winning_ticket(&amount);
    }

    #[only_owner]
    #[endpoint(setConfirmationPeriodStartEpoch)]
    fn set_confirmation_period_start_epoch(&self, new_start_epoch: u64) {
        self.configuration().update(|config| {
            self.require_valid_config_epoch_change(
                config.confirmation_period_start_epoch,
                new_start_epoch,
            );

            config.confirmation_period_start_epoch = new_start_epoch;
            self.require_valid_time_periods(config);
        });
    }

    #[only_owner]
    #[endpoint(setWinnerSelectionStartEpoch)]
    fn set_winner_selection_start_epoch(&self, new_start_epoch: u64) {
        self.configuration().update(|config| {
            self.require_valid_config_epoch_change(
                config.winner_selection_start_epoch,
                new_start_epoch,
            );

            config.winner_selection_start_epoch = new_start_epoch;
            self.require_valid_time_periods(config);
        });
    }

    #[only_owner]
    #[endpoint(setClaimStartEpoch)]
    fn set_claim_start_epoch(&self, new_start_epoch: u64) {
        self.configuration().update(|config| {
            self.require_valid_config_epoch_change(config.claim_start_epoch, new_start_epoch);

            config.claim_start_epoch = new_start_epoch;
            self.require_valid_time_periods(config);
        });
    }

    fn try_set_ticket_price(&self, token_id: EgldOrEsdtTokenIdentifier, amount: BigUint) {
        require!(
            token_id.is_egld() || token_id.is_valid_esdt_identifier(),
            "Invalid token ID"
        );
        require!(amount > 0, "Ticket price must be higher than 0");

        self.ticket_price()
            .set(&TokenAmountPair { token_id, amount });
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

    fn require_valid_config_epoch_change(&self, old_start_epoch: u64, new_start_epoch: u64) {
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            old_start_epoch > current_epoch,
            "Cannot change start epoch, it's either in progress or passed already"
        );
        require!(
            new_start_epoch > current_epoch,
            "Start epoch cannot be in the past"
        );
    }

    fn require_valid_time_periods(&self, config: &EpochsConfig) {
        require!(
            config.confirmation_period_start_epoch < config.winner_selection_start_epoch,
            "Winner selection start epoch must be after confirm start epoch"
        );
        require!(
            config.winner_selection_start_epoch <= config.claim_start_epoch,
            "Claim period must be after winner selection"
        );
    }
}
