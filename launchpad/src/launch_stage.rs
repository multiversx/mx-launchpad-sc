elrond_wasm::imports!();

#[derive(PartialEq, PartialOrd)]
pub enum LaunchStage {
    AddTickets,
    Confirm,
    WinnerSelection,
    Claim,
}

#[elrond_wasm::module]
pub trait LaunchStageModule {
    fn get_launch_stage(&self) -> LaunchStage {
        let current_epoch = self.blockchain().get_block_epoch();
        let confirmation_period_start_epoch = self.confirmation_period_start_epoch().get();

        if current_epoch < confirmation_period_start_epoch {
            return LaunchStage::AddTickets;
        }

        let winner_selection_start_epoch = self.winner_selection_start_epoch().get();
        if current_epoch < winner_selection_start_epoch {
            return LaunchStage::Confirm;
        }

        let winner_selection_started = self.has_winner_selection_process_started();
        let were_winners_selected = self.were_winners_selected();
        if winner_selection_started && !were_winners_selected {
            return LaunchStage::WinnerSelection;
        }

        let claim_start_epoch = self.claim_start_epoch().get();
        if winner_selection_start_epoch == claim_start_epoch
            && current_epoch == winner_selection_start_epoch
        {
            if were_winners_selected {
                return LaunchStage::Claim;
            }

            return LaunchStage::WinnerSelection;
        }
        if current_epoch >= winner_selection_start_epoch && current_epoch < claim_start_epoch {
            return LaunchStage::WinnerSelection;
        }

        LaunchStage::Claim
    }

    #[inline(always)]
    fn require_add_tickets_period(&self) -> SCResult<()> {
        require!(
            self.get_launch_stage() == LaunchStage::AddTickets,
            "Add tickets period has passed"
        );
        Ok(())
    }

    #[inline(always)]
    fn require_confirmation_period(&self) -> SCResult<()> {
        require!(
            self.get_launch_stage() == LaunchStage::Confirm,
            "Not in confirmation period"
        );
        Ok(())
    }

    #[inline(always)]
    fn require_before_winner_selection(&self) -> SCResult<()> {
        require!(
            self.get_launch_stage() < LaunchStage::WinnerSelection,
            "May only modify blacklist before winner selection"
        );
        Ok(())
    }

    #[inline(always)]
    fn require_winner_selection_period(&self) -> SCResult<()> {
        require!(
            self.get_launch_stage() == LaunchStage::WinnerSelection,
            "Not in winner selection period"
        );
        Ok(())
    }

    #[inline(always)]
    fn require_claim_period(&self) -> SCResult<()> {
        require!(
            self.get_launch_stage() == LaunchStage::Claim,
            "Not in claim period"
        );
        Ok(())
    }

    #[inline(always)]
    fn has_winner_selection_process_started(&self) -> bool {
        self.winner_selection_process_started().get()
    }

    #[inline(always)]
    fn were_tickets_filtered(&self) -> bool {
        self.tickets_filtered().get()
    }

    #[inline(always)]
    fn were_winners_selected(&self) -> bool {
        self.winners_selected().get()
    }

    // storage

    #[view(getConfirmationPeriodStartEpoch)]
    #[storage_mapper("confirmationPeriodStartEpoch")]
    fn confirmation_period_start_epoch(&self) -> SingleValueMapper<u64>;

    #[view(getWinnerSelectionStart)]
    #[storage_mapper("winnerSelectionStartEpoch")]
    fn winner_selection_start_epoch(&self) -> SingleValueMapper<u64>;

    #[view(getClaimStartEpoch)]
    #[storage_mapper("claimStartEpoch")]
    fn claim_start_epoch(&self) -> SingleValueMapper<u64>;

    // flags

    #[storage_mapper("winnerSelectionProcessStarted")]
    fn winner_selection_process_started(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("ticketsFiltered")]
    fn tickets_filtered(&self) -> SingleValueMapper<bool>;

    #[view(wereWinnersSelected)]
    #[storage_mapper("winnersSelected")]
    fn winners_selected(&self) -> SingleValueMapper<bool>;
}
