elrond_wasm::imports!();

const NR_TICKETS_BUNDLED_WITH_NFT_BUY: usize = 1;

#[elrond_wasm::module]
pub trait ConfirmNftModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::tickets::TicketsModule
    + launchpad_common::permissions::PermissionsModule
    + launchpad_common::user_interactions::UserInteractionsModule
    + launchpad_common::blacklist::BlacklistModule
    + launchpad_common::token_send::TokenSendModule
    + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
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

        let new_user = self.confirmed_nft_user_list().insert(caller.clone());
        require!(new_user, "Already confirmed NFT");

        let payment = self.call_value().egld_or_single_esdt();
        let nft_cost = self.nft_cost().get();
        require!(
            payment.token_identifier == nft_cost.token_identifier
                && payment.token_nonce == nft_cost.token_nonce
                && payment.amount >= nft_cost.amount,
            "Invalid payment"
        );

        let payment_amount_for_extra_ticket = &payment.amount - &nft_cost.amount;
        if payment_amount_for_extra_ticket > 0 {
            self.confirm_tickets(
                &caller,
                &payment.token_identifier,
                &payment_amount_for_extra_ticket,
                NR_TICKETS_BUNDLED_WITH_NFT_BUY,
            );
        }
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

    fn require_valid_cost(&self, cost: &EgldOrEsdtTokenPayment<Self::Api>) {
        if cost.token_identifier.is_egld() {
            require!(cost.token_nonce == 0, "EGLD token has no nonce");
        } else {
            require!(cost.token_identifier.is_valid(), "Invalid ESDT token ID");
        }

        require!(cost.amount > 0, "Cost may not be 0");
    }

    #[storage_mapper("confirmedNftUserList")]
    fn confirmed_nft_user_list(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getNftCost)]
    #[storage_mapper("nftCost")]
    fn nft_cost(&self) -> SingleValueMapper<EgldOrEsdtTokenPayment<Self::Api>>;

    #[storage_mapper("totalAvailableNfts")]
    fn total_available_nfts(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("claimableNftPayment")]
    fn claimable_nft_payment(&self) -> SingleValueMapper<BigUint>;
}
