multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::token_release::UnlockMilestone;

#[derive(TypeAbi, TopEncode)]
pub struct ClaimLaunchpadTokensEvent<M: ManagedTypeApi> {
    user: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    token_payment: EsdtTokenPayment<M>,
}

#[derive(TypeAbi, TopEncode)]
pub struct AddUsersToBlacklistEvent<M: ManagedTypeApi> {
    admin: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    users: ManagedVec<M, ManagedAddress<M>>,
}

#[derive(TypeAbi, TopEncode)]
pub struct RemoveGuaranteedUsersFromBlacklistEvent<M: ManagedTypeApi> {
    admin: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    users: ManagedVec<M, ManagedAddress<M>>,
}

#[derive(TypeAbi, TopEncode)]
pub struct SetUnlockScheduleEvent<M: ManagedTypeApi> {
    admin: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    milestones: ManagedVec<M, UnlockMilestone>,
}

#[derive(TypeAbi, TopEncode)]
pub struct AddTicketsEvent<M: ManagedTypeApi> {
    admin: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    users_count: usize,
    total_tickets_added: usize,
    total_guaranteed_tickets_added: usize,
}

#[derive(TypeAbi, TopEncode)]
pub struct DistributeGuaranteedTicketsCompletedEvent<M: ManagedTypeApi> {
    admin: ManagedAddress<M>,
    block: u64,
    epoch: u64,
    total_additional_winning_tickets: usize,
}

#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_claim_launchpad_tokens_event(&self, token_payment: EsdtTokenPayment) {
        let user = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.claim_launchpad_tokens_event(
            user.clone(),
            block,
            epoch,
            ClaimLaunchpadTokensEvent {
                user,
                block,
                epoch,
                token_payment,
            },
        )
    }

    fn emit_add_users_to_blacklist_event(&self, users: ManagedVec<ManagedAddress>) {
        let admin = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.add_users_to_blacklist_event(
            admin.clone(),
            block,
            epoch,
            AddUsersToBlacklistEvent {
                admin,
                block,
                epoch,
                users,
            },
        )
    }

    fn emit_remove_guaranteed_users_from_blacklist_event(&self, users: ManagedVec<ManagedAddress>) {
        let admin = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.remove_guaranteed_users_from_blacklist_event(
            admin.clone(),
            block,
            epoch,
            RemoveGuaranteedUsersFromBlacklistEvent {
                admin,
                block,
                epoch,
                users,
            },
        )
    }

    fn emit_set_unlock_schedule_event(&self, milestones: ManagedVec<UnlockMilestone>) {
        let admin = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.set_unlock_schedule_event(
            admin.clone(),
            block,
            epoch,
            SetUnlockScheduleEvent {
                admin,
                block,
                epoch,
                milestones,
            },
        )
    }

    fn emit_add_tickets_event(
        &self,
        users_count: usize,
        total_tickets_added: usize,
        total_guaranteed_tickets_added: usize,
    ) {
        let admin = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.add_tickets_event(
            admin.clone(),
            block,
            epoch,
            AddTicketsEvent {
                admin,
                block,
                epoch,
                users_count,
                total_tickets_added,
                total_guaranteed_tickets_added,
            },
        )
    }

    fn emit_distribute_guaranteed_tickets_completed_event(
        &self,
        total_additional_winning_tickets: usize,
    ) {
        let admin = self.blockchain().get_caller();
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        self.distribute_guaranteed_tickets_completed_event(
            admin.clone(),
            block,
            epoch,
            DistributeGuaranteedTicketsCompletedEvent {
                admin,
                block,
                epoch,
                total_additional_winning_tickets,
            },
        )
    }

    #[event("claimLaunchpadTokens")]
    fn claim_launchpad_tokens_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        claim_launchpad_tokens_event: ClaimLaunchpadTokensEvent<Self::Api>,
    );

    #[event("addUsersToBlacklist")]
    fn add_users_to_blacklist_event(
        &self,
        #[indexed] admin: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        add_users_to_blacklist_event: AddUsersToBlacklistEvent<Self::Api>,
    );

    #[event("removeGuaranteedUsersFromBlacklist")]
    fn remove_guaranteed_users_from_blacklist_event(
        &self,
        #[indexed] admin: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        remove_guaranteed_users_from_blacklist_event: RemoveGuaranteedUsersFromBlacklistEvent<
            Self::Api,
        >,
    );

    #[event("setUnlockSchedule")]
    fn set_unlock_schedule_event(
        &self,
        #[indexed] admin: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        set_unlock_schedule_event: SetUnlockScheduleEvent<Self::Api>,
    );

    #[event("addTickets")]
    fn add_tickets_event(
        &self,
        #[indexed] admin: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        add_tickets_event: AddTicketsEvent<Self::Api>,
    );

    #[event("distributeGuaranteedTicketsCompleted")]
    fn distribute_guaranteed_tickets_completed_event(
        &self,
        #[indexed] admin: ManagedAddress,
        #[indexed] block: u64,
        #[indexed] epoch: u64,
        distribute_guaranteed_tickets_completed_event: DistributeGuaranteedTicketsCompletedEvent<
            Self::Api,
        >,
    );
}
