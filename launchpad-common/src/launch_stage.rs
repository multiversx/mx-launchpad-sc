multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::config::TimelineConfig;

#[derive(PartialEq, PartialOrd)]
pub enum LaunchStage {
    AddTickets,
    Confirm,
    WinnerSelection,
    Claim,
}

#[derive(TypeAbi, TopEncode, TopDecode, Default)]
pub struct Flags {
    pub has_winner_selection_process_started: bool,
    pub were_tickets_filtered: bool,
    pub were_winners_selected: bool,
    pub was_additional_step_completed: bool,
}

#[multiversx_sc::module]
pub trait LaunchStageModule: crate::config::ConfigModule {
    fn get_launch_stage(&self) -> LaunchStage {
        let current_round = self.blockchain().get_block_round();
        let config: TimelineConfig = self.configuration().get();
        let flags: Flags = self.flags().get();

        if current_round < config.confirmation_period_start_round {
            return LaunchStage::AddTickets;
        }
        if current_round < config.winner_selection_start_round {
            return LaunchStage::Confirm;
        }

        let both_selection_steps_completed =
            flags.were_winners_selected && flags.was_additional_step_completed;
        if current_round >= config.winner_selection_start_round && !both_selection_steps_completed {
            return LaunchStage::WinnerSelection;
        }
        if current_round >= config.winner_selection_start_round
            && current_round < config.claim_start_round
        {
            return LaunchStage::WinnerSelection;
        }

        LaunchStage::Claim
    }

    #[inline]
    fn require_add_tickets_period(&self) {
        require!(
            self.get_launch_stage() == LaunchStage::AddTickets,
            "Add tickets period has passed"
        );
    }

    #[inline]
    fn require_confirmation_period(&self) {
        require!(
            self.get_launch_stage() == LaunchStage::Confirm,
            "Not in confirmation period"
        );
    }

    #[inline]
    fn require_before_winner_selection(&self) {
        require!(
            self.get_launch_stage() < LaunchStage::WinnerSelection,
            "May only modify blacklist before winner selection"
        );
    }

    #[inline]
    fn require_winner_selection_period(&self) {
        require!(
            self.get_launch_stage() == LaunchStage::WinnerSelection,
            "Not in winner selection period"
        );
    }

    #[inline]
    fn require_claim_period(&self) {
        require!(
            self.get_launch_stage() == LaunchStage::Claim,
            "Not in claim period"
        );
    }

    #[view(getLaunchStageFlags)]
    #[storage_mapper("flags")]
    fn flags(&self) -> SingleValueMapper<Flags>;
}
