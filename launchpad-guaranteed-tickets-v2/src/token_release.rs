multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use launchpad_common::config;

pub const MAX_PERCENTAGE: u64 = 10_000;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, ManagedVecItem)]
pub struct UnlockMilestone {
    pub release_epoch: u64,
    pub percentage: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi, NestedEncode, NestedDecode)]
pub struct UnlockSchedule<M: ManagedTypeApi> {
    milestones: ManagedVec<M, UnlockMilestone>,
}

impl<M: ManagedTypeApi> UnlockSchedule<M> {
    pub fn new(milestones: ManagedVec<M, UnlockMilestone>) -> Self {
        UnlockSchedule { milestones }
    }

    fn validate(&self) -> bool {
        if self.milestones.is_empty() {
            return false;
        }

        let mut total_percentage = 0u64;
        let mut last_epoch = 0u64;

        for milestone in self.milestones.iter() {
            if milestone.release_epoch < last_epoch || milestone.percentage > MAX_PERCENTAGE {
                return false;
            }
            last_epoch = milestone.release_epoch;
            total_percentage += milestone.percentage;
        }

        total_percentage == MAX_PERCENTAGE
    }
}

#[multiversx_sc::module]
pub trait TokenReleaseModule: config::ConfigModule + crate::events::EventsModule {
    #[only_owner]
    #[endpoint(setUnlockSchedule)]
    fn set_unlock_schedule(&self, unlock_milestones: MultiValueEncoded<UnlockMilestone>) {
        let configuration = self.configuration();
        require!(
            !configuration.is_empty(),
            "Timeline configuration is not set"
        );
        let confirmation_period_start_block = configuration.get().confirmation_period_start_block;

        let current_block = self.blockchain().get_block_nonce();
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            current_block < confirmation_period_start_block || self.unlock_schedule().is_empty(),
            "Can't change the unlock schedule"
        );

        let milestones = unlock_milestones.to_vec();
        let unlock_schedule = UnlockSchedule::new(milestones.clone());
        require!(unlock_schedule.validate(), "Invalid unlock schedule");

        require!(
            unlock_schedule.milestones.get(0).release_epoch >= current_epoch,
            "First milestone epoch must be in the future"
        );

        self.unlock_schedule().set(unlock_schedule);

        self.emit_set_unlock_schedule_event(milestones);
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
        let current_epoch = self.blockchain().get_block_epoch();

        let mut claimable_percentage = 0u64;
        for milestone in unlock_schedule.milestones.iter() {
            if milestone.release_epoch <= current_epoch {
                claimable_percentage += milestone.percentage;
            } else {
                break;
            }
        }

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
    fn unlock_schedule(&self) -> SingleValueMapper<UnlockSchedule<Self::Api>>;
}
