multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use launchpad_common::{config, launch_stage};

pub const MAX_PERCENTAGE: u64 = 10_000;
pub const MAX_UNLOCK_MILESTONES_ENTRIES: usize = 60;
pub const MAX_RELEASE_EPOCH_DIFF: u64 = 1800;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, ManagedVecItem)]
pub struct UnlockMilestone {
    pub release_epoch: u64,
    pub percentage: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi, NestedEncode, NestedDecode)]
pub struct UnlockSchedule<M: ManagedTypeApi> {
    milestones: ManagedVec<M, UnlockMilestone>,
}

impl<M: ManagedTypeApi> Default for UnlockSchedule<M> {
    fn default() -> Self {
        Self {
            milestones: ManagedVec::from_single_item(UnlockMilestone {
                release_epoch: 0,
                percentage: MAX_PERCENTAGE,
            }),
        }
    }
}

impl<M: ManagedTypeApi> UnlockSchedule<M> {
    pub fn new(milestones: ManagedVec<M, UnlockMilestone>) -> Self {
        UnlockSchedule { milestones }
    }

    fn validate(&self, current_epoch: u64) -> bool {
        if self.milestones.is_empty() {
            return false;
        }

        let mut total_percentage = 0u64;
        let mut last_epoch = 0u64;

        for milestone in self.milestones.iter() {
            if milestone.percentage > MAX_PERCENTAGE
                || milestone.release_epoch < current_epoch
                || milestone.release_epoch < last_epoch
                || milestone.release_epoch > current_epoch + MAX_RELEASE_EPOCH_DIFF
            {
                return false;
            }

            last_epoch = milestone.release_epoch;
            total_percentage += milestone.percentage;
        }

        total_percentage == MAX_PERCENTAGE
    }
}

#[multiversx_sc::module]
pub trait TokenReleaseModule:
    config::ConfigModule + launch_stage::LaunchStageModule + crate::events::EventsModule
{
    #[only_owner]
    #[endpoint(setUnlockSchedule)]
    fn set_unlock_schedule(&self, unlock_milestones: MultiValueEncoded<MultiValue2<u64, u64>>) {
        self.require_add_tickets_period();
        require!(
            unlock_milestones.len() <= MAX_UNLOCK_MILESTONES_ENTRIES,
            "Maximum unlock milestones entries exceeded"
        );

        let mut milestones = ManagedVec::new();
        for unlock_milestone in unlock_milestones {
            let (release_epoch, percentage) = unlock_milestone.into_tuple();
            milestones.push(UnlockMilestone {
                release_epoch,
                percentage,
            });
        }

        let current_epoch = self.blockchain().get_block_epoch();
        let unlock_schedule = UnlockSchedule::new(milestones.clone());
        require!(
            unlock_schedule.validate(current_epoch),
            "Invalid unlock schedule"
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
        let unlock_schedule = if unlock_schedule_mapper.is_empty() {
            UnlockSchedule::default()
        } else {
            unlock_schedule_mapper.get()
        };

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
