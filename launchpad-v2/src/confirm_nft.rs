elrond_wasm::imports!();

use elrond_wasm::storage::StorageKey;

#[elrond_wasm::module]
pub trait ConfirmNftModule:
    launchpad_common::launch_stage::LaunchStageModule + launchpad_common::config::ConfigModule
{
    #[payable("*")]
    #[endpoint(confirmNft)]
    fn confirm_nft(&self) {
        self.require_confirmation_period();

        let caller = self.blockchain().get_caller();
        let new_user = self.confirmed_nft_user_list().insert(caller);
        require!(new_user, "Already confirmed NFT");

        let payment = self.call_value().payment();
        self.require_exact_nft_cost(&payment);
    }

    fn require_exact_nft_cost(&self, payment: &EsdtTokenPayment<Self::Api>) {
        let nft_cost = self.nft_cost().get();
        require!(
            payment.token_identifier == nft_cost.token_identifier
                && payment.token_nonce == nft_cost.token_nonce
                && payment.amount == nft_cost.amount,
            "Invalid payment"
        );
    }

    fn confirmed_list_to_vec_mapper(&self) -> VecMapper<ManagedAddress> {
        VecMapper::new(StorageKey::new(b"confirmedNftUserList"))
    }

    #[storage_mapper("confirmedNftUserList")]
    fn confirmed_nft_user_list(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("nftCost")]
    fn nft_cost(&self) -> SingleValueMapper<EsdtTokenPayment<Self::Api>>;

    #[storage_mapper("totalAvailableNfts")]
    fn total_available_nfts(&self) -> SingleValueMapper<u64>;
}
