elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(PartialEq, PartialOrd)]
pub enum LaunchStage {
    AddTickets,
    Confirm,
    WinnerSelection,
    Claim,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct EpochsConfig {
    pub confirmation_period_start_epoch: u64,
    pub winner_selection_start_epoch: u64,
    pub claim_start_epoch: u64,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct Flags {
    pub has_winner_selection_process_started: bool,
    pub were_tickets_filtered: bool,
    pub were_winners_selected: bool,
}

#[elrond_wasm::module]
pub trait LaunchStageModule {
    fn get_launch_stage(&self) -> LaunchStage {
        let current_epoch = self.blockchain().get_block_epoch();
        let config: EpochsConfig = self.configuration().get();
        let flags: Flags = self.flags().get();

        if current_epoch < config.confirmation_period_start_epoch {
            return LaunchStage::AddTickets;
        }
        if current_epoch < config.winner_selection_start_epoch {
            return LaunchStage::Confirm;
        }
        if flags.has_winner_selection_process_started && !flags.were_winners_selected {
            return LaunchStage::WinnerSelection;
        }
        if config.winner_selection_start_epoch == config.claim_start_epoch
            && current_epoch == config.winner_selection_start_epoch
        {
            if flags.were_winners_selected {
                return LaunchStage::Claim;
            }

            return LaunchStage::WinnerSelection;
        }
        if current_epoch >= config.winner_selection_start_epoch
            && current_epoch < config.claim_start_epoch
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

    #[view(getConfiguration)]
    #[storage_mapper("configuration")]
    fn configuration(&self) -> SingleValueMapper<EpochsConfig>;

    #[view(getLaunchStageFlags)]
    #[storage_mapper("flags")]
    fn flags(&self) -> SingleValueMapper<Flags>;
}
