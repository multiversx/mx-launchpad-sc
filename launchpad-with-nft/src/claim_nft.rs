use crate::mystery_sft::{MysterySftTypes, NFT_AMOUNT};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ClaimNftModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::blacklist::BlacklistModule
    + launchpad_common::tickets::TicketsModule
    + launchpad_common::token_send::TokenSendModule
    + launchpad_common::permissions::PermissionsModule
    + launchpad_common::user_interactions::UserInteractionsModule
    + launchpad_common::ongoing_operation::OngoingOperationModule
    + launchpad_common::common_events::CommonEventsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + multiversx_sc_modules::pause::PauseModule
    + crate::nft_config::NftConfigModule
    + crate::mystery_sft::MysterySftModule
    + crate::confirm_nft::ConfirmNftModule
    + crate::nft_winners_selection::NftWinnersSelectionModule
{
    fn claim_nft(&self) {
        let caller = self.blockchain().get_caller();
        let mystery_sft_type = if self.nft_selection_winners().swap_remove(&caller) {
            MysterySftTypes::ConfirmedWon
        } else if self.confirmed_nft_user_list().swap_remove(&caller) {
            MysterySftTypes::ConfirmedLost
        } else {
            MysterySftTypes::NotConfirmed
        };

        let _ = self.mystery_sft().nft_add_quantity_and_send(
            &caller,
            mystery_sft_type.as_nonce(),
            NFT_AMOUNT.into(),
        );

        if matches!(mystery_sft_type, MysterySftTypes::ConfirmedLost) {
            let nft_cost = self.nft_cost().get();
            self.send().direct(
                &caller,
                &nft_cost.token_identifier,
                nft_cost.token_nonce,
                &nft_cost.amount,
            );
        }
    }
}
