multiversx_sc::imports!();

use crate::config::{TimelineConfig, TokenAmountPair};

#[multiversx_sc::module]
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

        self.launchpad_tokens_deposited().set(true);
        self.total_launchpad_tokens_deposited().set(payment_amount);
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
    #[endpoint(setConfirmationPeriodStartBlock)]
    fn set_confirmation_period_start_block(&self, new_start_block: u64) {
        self.configuration().update(|config| {
            self.require_valid_config_timeline_change(
                config.confirmation_period_start_block,
                new_start_block,
            );

            config.confirmation_period_start_block = new_start_block;
            self.require_valid_time_periods(config);
        });
    }

    #[only_owner]
    #[endpoint(setWinnerSelectionStartBlock)]
    fn set_winner_selection_start_block(&self, new_start_block: u64) {
        self.configuration().update(|config| {
            self.require_valid_config_timeline_change(
                config.winner_selection_start_block,
                new_start_block,
            );

            config.winner_selection_start_block = new_start_block;
            self.require_valid_time_periods(config);
        });
    }

    #[only_owner]
    #[endpoint(setClaimStartBlock)]
    fn set_claim_start_block(&self, new_start_block: u64) {
        self.configuration().update(|config| {
            self.require_valid_config_timeline_change(config.claim_start_block, new_start_block);

            config.claim_start_block = new_start_block;
            self.require_valid_time_periods(config);
        });
    }

    fn try_set_ticket_price(&self, token_id: EgldOrEsdtTokenIdentifier, amount: BigUint) {
        require!(token_id.is_valid(), "Invalid token ID");
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

        self.nr_winning_tickets().set(nr_winning_tickets);
    }

    fn require_valid_config_timeline_change(&self, old_start_block: u64, new_start_block: u64) {
        let current_block = self.blockchain().get_block_nonce();
        require!(
            old_start_block > current_block,
            "Cannot change start block, it's either in progress or passed already"
        );
        require!(
            new_start_block > current_block,
            "Start block cannot be in the past"
        );
    }

    fn require_valid_time_periods(&self, config: &TimelineConfig) {
        require!(
            config.confirmation_period_start_block < config.winner_selection_start_block,
            "Winner selection start block must be after confirm start block"
        );
        require!(
            config.winner_selection_start_block <= config.claim_start_block,
            "Claim period must be after winner selection"
        );
    }
}
