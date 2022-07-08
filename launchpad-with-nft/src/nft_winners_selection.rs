elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use launchpad_common::{
    ongoing_operation::{CONTINUE_OP, STOP_OP},
    random::Random,
};

const VEC_MAPPER_START_INDEX: usize = 1;

#[elrond_wasm::module]
pub trait NftWinnersSelectionModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::ongoing_operation::OngoingOperationModule
    + launchpad_common::tickets::TicketsModule
    + launchpad_common::permissions::PermissionsModule
    + launchpad_common::user_interactions::UserInteractionsModule
    + launchpad_common::blacklist::BlacklistModule
    + launchpad_common::token_send::TokenSendModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + crate::confirm_nft::ConfirmNftModule
    + crate::mystery_sft::MysterySftModule
{
    fn select_nft_winners(&self, rng: &mut Random<Self::Api>) -> OperationCompletionStatus {
        let mut all_users_mapper = self.confirmed_nft_user_list();
        let mut nft_winners_mapper = self.nft_selection_winners();

        let mut users_left = all_users_mapper.len();
        let mut winners_selected = nft_winners_mapper.len();
        let total_available_nfts = self.total_available_nfts().get();

        self.run_while_it_has_gas(|| {
            if users_left == 0 || winners_selected == total_available_nfts {
                return STOP_OP;
            }

            let rand_index = rng.next_usize_in_range(VEC_MAPPER_START_INDEX, users_left + 1);
            let winner_addr = all_users_mapper.get_by_index(rand_index);

            all_users_mapper.swap_remove(&winner_addr);
            let _ = nft_winners_mapper.insert(winner_addr);

            users_left -= 1;
            winners_selected += 1;

            CONTINUE_OP
        })
    }

    #[view(hasUserConfirmedNft)]
    fn has_user_confirmed_nft(&self, user: ManagedAddress) -> bool {
        self.confirmed_nft_user_list().contains(&user)
            || self.nft_selection_winners().contains(&user)
    }

    #[view(hasUserWonNft)]
    fn has_user_won_nft(&self, user: ManagedAddress) -> bool {
        self.nft_selection_winners().contains(&user)
    }

    #[storage_mapper("nftSelectionWinners")]
    fn nft_selection_winners(&self) -> UnorderedSetMapper<ManagedAddress>;
}
