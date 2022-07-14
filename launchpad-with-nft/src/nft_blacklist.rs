elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait NftBlacklistModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::tickets::TicketsModule
    + launchpad_common::permissions::PermissionsModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + crate::nft_config::NftConfigModule
    + crate::confirm_nft::ConfirmNftModule
    + crate::mystery_sft::MysterySftModule
{
    fn refund_nft_cost_after_blacklist(&self, users: &ManagedVec<ManagedAddress>) {
        let nft_cost = self.nft_cost().get();
        for user in users {
            let did_user_confirm = self.confirmed_nft_user_list().swap_remove(&user);
            if did_user_confirm {
                self.send().direct(
                    &user,
                    &nft_cost.token_identifier,
                    nft_cost.token_nonce,
                    &nft_cost.amount,
                );
            }
        }
    }
}
