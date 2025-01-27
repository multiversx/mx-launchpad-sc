multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use launchpad_common::config;

pub const MAX_PERCENTAGE: u64 = 10_000;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct UnlockSchedule {
    claim_start_round: u64,
    initial_release_percentage: u64,
    vesting_release_times: u64,
    vesting_release_percentage: u64,
    vesting_release_period: u64,
}

impl UnlockSchedule {
    pub fn new(
        claim_start_round: u64,
        initial_release_percentage: u64,
        vesting_release_times: u64,
        vesting_release_percentage: u64,
        vesting_release_period: u64,
    ) -> Self {
        UnlockSchedule {
            claim_start_round,
            initial_release_percentage,
            vesting_release_times,
            vesting_release_percentage,
            vesting_release_period,
        }
    }
}

#[multiversx_sc::module]
pub trait TokenReleaseModule: config::ConfigModule {
    #[only_owner]
    #[endpoint(setUnlockSchedule)]
    fn set_unlock_schedule(
        &self,
        claim_start_round: u64,
        initial_release_percentage: u64,
        vesting_release_times: u64,
        vesting_release_percentage: u64,
        vesting_release_period: u64,
    ) {
        let configuration = self.configuration();
        require!(
            !configuration.is_empty(),
            "Timeline configuration is not set"
        );
        let confirmation_period_start_round = configuration.get().confirmation_period_start_round;

        let current_round = self.blockchain().get_block_round();
        require!(
            current_round < confirmation_period_start_round || self.unlock_schedule().is_empty(),
            "Can't change the unlock schedule"
        );
        require!(
            claim_start_round >= current_round,
            "Wrong claim start round"
        );
        require!(
            vesting_release_period > 0 || initial_release_percentage == MAX_PERCENTAGE,
            "Wrong vesting release recurrency"
        );

        let unlock_percentage =
            initial_release_percentage + vesting_release_times * vesting_release_percentage;

        require!(
            unlock_percentage == MAX_PERCENTAGE,
            "Unlock percentage is not 100%"
        );

        let unlock_schedule = UnlockSchedule::new(
            claim_start_round,
            initial_release_percentage,
            vesting_release_times,
            vesting_release_percentage,
            vesting_release_period,
        );

        self.unlock_schedule().set(unlock_schedule);
    }

    #[view(getClaimableTokens)]
    fn compute_claimable_tokens(&self, address: &ManagedAddress) -> BigUint {
        let user_total_claimable_balance = self.user_total_claimable_balance(address).get();
        if user_total_claimable_balance == 0 {
            return BigUint::zero();
        }

        let user_claimed_balance = self.user_claimed_balance(address).get();
        require!(
            user_claimed_balance < user_total_claimable_balance,
            "Already claimed all tokens"
        );

        let unlock_schedule_mapper = self.unlock_schedule();
        if unlock_schedule_mapper.is_empty() {
            return BigUint::zero();
        }
        let unlock_schedule = unlock_schedule_mapper.get();
        let current_round = self.blockchain().get_block_round();
        if unlock_schedule.claim_start_round > current_round {
            return BigUint::zero();
        }

        if unlock_schedule.initial_release_percentage == MAX_PERCENTAGE {
            return user_total_claimable_balance;
        }

        let rounds_passed = current_round - unlock_schedule.claim_start_round;
        let mut claimable_periods = rounds_passed / unlock_schedule.vesting_release_period;
        if claimable_periods > unlock_schedule.vesting_release_times {
            claimable_periods = unlock_schedule.vesting_release_times;
        }
        let claimable_percentage = unlock_schedule.initial_release_percentage
            + unlock_schedule.vesting_release_percentage * claimable_periods;
        let current_claimable_tokens =
            &user_total_claimable_balance * claimable_percentage / MAX_PERCENTAGE;

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
    fn unlock_schedule(&self) -> SingleValueMapper<UnlockSchedule>;
}
