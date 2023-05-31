multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ConfirmNftModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::tickets::TicketsModule
    + launchpad_common::permissions::PermissionsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + crate::nft_config::NftConfigModule
    + crate::mystery_sft::MysterySftModule
{
    #[payable("*")]
    #[endpoint(confirmNft)]
    fn confirm_nft(&self) {
        self.require_confirmation_period();
        self.require_all_sft_setup_steps_complete();

        let caller = self.blockchain().get_caller();
        let nr_base_launchpad_confirmed = self.nr_confirmed_tickets(&caller).get();
        require!(
            nr_base_launchpad_confirmed > 0,
            "Must confirm launchpad tickets before entering NFT draw"
        );

        let new_user = self.confirmed_nft_user_list().insert(caller);
        require!(new_user, "Already confirmed NFT");

        let payment = self.call_value().egld_or_single_esdt();
        self.require_exact_nft_cost(&payment);
    }

    fn claim_nft_payment(&self) {
        self.require_claim_period();

        let mapper = self.claimable_nft_payment();
        let claimable_amount = mapper.get();
        if claimable_amount > 0 {
            let mut payment = self.nft_cost().get();
            payment.amount = claimable_amount;

            let owner = self.blockchain().get_caller();
            self.send().direct(
                &owner,
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            );

            mapper.clear();
        }
    }

    fn require_exact_nft_cost(&self, payment: &EgldOrEsdtTokenPayment<Self::Api>) {
        let nft_cost = self.nft_cost().get();
        require!(
            payment.token_identifier == nft_cost.token_identifier
                && payment.token_nonce == nft_cost.token_nonce
                && payment.amount == nft_cost.amount,
            "Invalid payment"
        );
    }

    #[storage_mapper("confirmedNftUserList")]
    fn confirmed_nft_user_list(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("totalAvailableNfts")]
    fn total_available_nfts(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("claimableNftPayment")]
    fn claimable_nft_payment(&self) -> SingleValueMapper<BigUint>;
}
