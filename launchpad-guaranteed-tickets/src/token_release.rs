multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use launchpad_common::config;

pub const MAX_PERCENTAGE: u64 = 10_000;
pub const MAX_VESTING_RELEASES: usize = 50;

#[derive(TypeAbi, ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct VestingRelease {
    vesting_release_epoch: u64,
    vesting_release_percentage: u64,
}

impl VestingRelease {
    pub fn new(vesting_release_epoch: u64, vesting_release_percentage: u64) -> Self {
        VestingRelease {
            vesting_release_epoch,
            vesting_release_percentage,
        }
    }
}

#[multiversx_sc::module]
pub trait TokenReleaseModule: config::ConfigModule {
    #[only_owner]
    #[endpoint(setUnlockSchedule)]
    fn set_unlock_schedule(&self, vesting_releases: MultiValueEncoded<MultiValue2<u64, u64>>) {
        let configuration = self.configuration();
        require!(
            !configuration.is_empty(),
            "Timeline configuration is not set"
        );
        let confirmation_period_start_block = configuration.get().confirmation_period_start_block;

        let current_block = self.blockchain().get_block_nonce();
        require!(
            current_block < confirmation_period_start_block || self.unlock_schedule().is_empty(),
            "Can't change the unlock schedule"
        );
        require!(
            !vesting_releases.is_empty() && vesting_releases.len() <= MAX_VESTING_RELEASES,
            "Wrong release schedule"
        );

        let mut unlock_schedule = ManagedVec::new();
        let mut total_unlock_percentage = 0;
        let mut last_release_epoch = self.blockchain().get_block_epoch();

        for vesting_release in vesting_releases {
            let (vesting_release_epoch, vesting_release_percentage) = vesting_release.into_tuple();

            require!(
                vesting_release_epoch >= last_release_epoch,
                "The release epochs must be in order"
            );

            total_unlock_percentage += vesting_release_percentage;
            last_release_epoch = vesting_release_epoch;

            unlock_schedule.push(VestingRelease::new(
                vesting_release_epoch,
                vesting_release_percentage,
            ));
        }

        require!(
            total_unlock_percentage == MAX_PERCENTAGE,
            "Unlock percentage is not 100%"
        );

        self.unlock_schedule().set(unlock_schedule);
    }

    #[view(getClaimableTokens)]
    fn compute_claimable_tokens(&self, address: &ManagedAddress) -> BigUint {
        let user_total_claimable_balance = self.user_total_claimable_balance(address).get();
        let user_claimed_balance = self.user_claimed_balance(address).get();
        require!(
            user_total_claimable_balance > 0,
            "User has no claimable tokens"
        );
        require!(
            user_claimed_balance < user_total_claimable_balance,
            "Already claimed all tokens"
        );

        let unlock_schedule_mapper = self.unlock_schedule();
        if unlock_schedule_mapper.is_empty() {
            return BigUint::zero();
        }
        let unlock_schedule = unlock_schedule_mapper.get();
        let current_epoch = self.blockchain().get_block_epoch();
        let mut current_claimable_percentage = 0;
        for vesting_release in unlock_schedule.iter() {
            if vesting_release.vesting_release_epoch < current_epoch {
                break;
            }

            current_claimable_percentage += vesting_release.vesting_release_percentage;
        }

        let current_claimable_tokens =
            &user_total_claimable_balance * current_claimable_percentage / MAX_PERCENTAGE;

        current_claimable_tokens - user_claimed_balance
    }

    #[view(getUserTotalClaimableBalance)]
    #[storage_mapper("userTotalClaimableBalance")]
    fn user_total_claimable_balance(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[view(getUserClaimedBalance)]
    #[storage_mapper("userClaimedBalance")]
    fn user_claimed_balance(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[view(getUnlockSchedule)]
    #[storage_mapper("unlockSchedule")]
    fn unlock_schedule(&self) -> SingleValueMapper<ManagedVec<VestingRelease>>;
}
