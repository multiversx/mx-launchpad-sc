elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait LaunchStageModule {
    fn require_add_tickets_period(&self) -> SCResult<()> {
        let current_epoch = self.blockchain().get_block_epoch();
        let confirmation_period_start_epoch = self.confirmation_period_start_epoch().get();

        require!(
            current_epoch < confirmation_period_start_epoch,
            "Add tickets period has passed"
        );
        Ok(())
    }

    fn require_confirmation_period(&self) -> SCResult<()> {
        let current_epoch = self.blockchain().get_block_epoch();
        let confirmation_period_start_epoch = self.confirmation_period_start_epoch().get();
        let winner_selection_start_epoch = self.winner_selection_start_epoch().get();

        require!(
            current_epoch >= confirmation_period_start_epoch
                && current_epoch < winner_selection_start_epoch,
            "Not in confirmation period"
        );
        Ok(())
    }

    fn require_before_winner_selection(&self) -> SCResult<()> {
        let current_epoch = self.blockchain().get_block_epoch();
        let winner_selection_start_epoch = self.winner_selection_start_epoch().get();

        require!(
            current_epoch < winner_selection_start_epoch,
            "May only modify blacklist before winner selection"
        );
        Ok(())
    }

    fn require_winner_selection_period(&self) -> SCResult<()> {
        let current_epoch = self.blockchain().get_block_epoch();
        let winner_selection_start_epoch = self.winner_selection_start_epoch().get();
        let claim_start_epoch = self.claim_start_epoch().get();

        if winner_selection_start_epoch == claim_start_epoch {
            require!(
                current_epoch == winner_selection_start_epoch,
                "Not in winner selection period"
            );
        } else {
            require!(
                current_epoch >= winner_selection_start_epoch && current_epoch < claim_start_epoch,
                "Not in winner selection period"
            );
        }

        Ok(())
    }

    fn require_claim_period(&self, winners_selected: bool) -> SCResult<()> {
        let current_epoch = self.blockchain().get_block_epoch();
        let mut claim_start_epoch = self.claim_start_epoch().get();
        if !winners_selected {
            claim_start_epoch += 1;
        }

        require!(current_epoch >= claim_start_epoch, "Not in claim period");
        Ok(())
    }

    // storage

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
