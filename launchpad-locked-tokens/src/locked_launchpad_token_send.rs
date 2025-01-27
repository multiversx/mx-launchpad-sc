multiversx_sc::imports!();

const MAX_PERCENTAGE: u32 = 10_000; // 100%

pub mod simple_lock_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait SimpleLockProxy {
        #[payable("*")]
        #[endpoint(lockTokens)]
        fn lock_tokens(&self, unlock_epoch: u64, destination: ManagedAddress);
    }
}

#[multiversx_sc::module]
pub trait LockedLaunchpadTokenSend {
    fn try_set_launchpad_tokens_lock_percentage(&self, lock_percentage: u32) {
        require!(
            lock_percentage > 0 && lock_percentage <= MAX_PERCENTAGE,
            "Invalid lock percentage"
        );

        self.launchpad_tokens_lock_percentage().set(lock_percentage);
    }

    fn try_set_launchpad_tokens_unlock_epoch(&self, unlock_epoch: u64) {
        let current_epoch = self.blockchain().get_block_epoch();
        require!(unlock_epoch > current_epoch, "Invalid unlock epoch");

        self.launchpad_tokens_unlock_epoch().set(unlock_epoch);
    }

    fn try_set_simple_lock_sc_address(&self, sc_address: ManagedAddress) {
        require!(
            !sc_address.is_zero() && self.blockchain().is_smart_contract(&sc_address),
            "Invalid SC address"
        );

        self.simple_lock_sc_address().set(&sc_address);
    }

    fn send_locked_launchpad_tokens(
        &self,
        dest_address: &ManagedAddress,
        launchpad_tokens: &EsdtTokenPayment,
    ) {
        let mut unlocked_amount = launchpad_tokens.amount.clone();

        let unlock_epoch = self.launchpad_tokens_unlock_epoch().get();
        let current_epoch = self.blockchain().get_block_epoch();
        if current_epoch < unlock_epoch {
            let lock_percentage = self.launchpad_tokens_lock_percentage().get();
            let lock_amount = &launchpad_tokens.amount * lock_percentage / MAX_PERCENTAGE;
            if lock_amount > 0 {
                unlocked_amount -= &lock_amount;

                let sc_address = self.simple_lock_sc_address().get();
                let _: IgnoreValue = self
                    .simple_lock_proxy_builder(sc_address)
                    .lock_tokens(unlock_epoch, dest_address.clone())
                    .with_esdt_transfer((
                        launchpad_tokens.token_identifier.clone(),
                        launchpad_tokens.token_nonce,
                        lock_amount,
                    ))
                    .execute_on_dest_context();
            }
        }

        if unlocked_amount > 0 {
            self.send().direct_esdt(
                dest_address,
                &launchpad_tokens.token_identifier,
                launchpad_tokens.token_nonce,
                &unlocked_amount,
            );
        }
    }

    #[view(getLaunchpadTokensLockPercentage)]
    #[storage_mapper("launchpadTokensLockPercentage")]
    fn launchpad_tokens_lock_percentage(&self) -> SingleValueMapper<u32>;

    #[view(getLaunchpadTokensUnlockEpoch)]
    #[storage_mapper("launchpadTokensUnlockEpoch")]
    fn launchpad_tokens_unlock_epoch(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("simpleLockScAddress")]
    fn simple_lock_sc_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn simple_lock_proxy_builder(
        &self,
        sc_address: ManagedAddress,
    ) -> simple_lock_proxy::Proxy<Self::Api>;
}
