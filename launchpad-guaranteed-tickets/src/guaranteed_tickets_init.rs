multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const STAKING_GUARANTEED_TICKETS_NO: usize = 1;
pub const MIGRATION_GUARANTEED_TICKETS_NO: usize = 1;

#[derive(TopEncode, TopDecode)]
pub struct UserTicketsStatus {
    pub staking_tickets_allowance: usize,
    pub energy_tickets_allowance: usize,
    pub staking_guaranteed_tickets: usize,
    pub migration_guaranteed_tickets: usize,
}

impl UserTicketsStatus {
    fn new(staking_tickets_allowance: usize, energy_tickets_allowance: usize) -> Self {
        Self {
            staking_tickets_allowance,
            energy_tickets_allowance,
            staking_guaranteed_tickets: 0usize,
            migration_guaranteed_tickets: 0usize,
        }
    }
}

#[multiversx_sc::module]
pub trait GuaranteedTicketsInitModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::ongoing_operation::OngoingOperationModule
    + launchpad_common::tickets::TicketsModule
{
    fn add_tickets_with_guaranteed_winners(
        &self,
        address_number_pairs: MultiValueEncoded<MultiValue4<ManagedAddress, usize, usize, bool>>,
    ) {
        self.require_add_tickets_period();

        let min_confirmed_for_guaranteed_ticket = self.min_confirmed_for_guaranteed_ticket().get();
        let mut guranteed_ticket_whitelist = self.users_with_guaranteed_ticket();
        let mut total_winning_tickets = self.nr_winning_tickets().get();
        let mut total_guaranteed_tickets = self.total_guaranteed_tickets().get();

        for multi_arg in address_number_pairs {
            let (buyer, nr_staking_tickets, nr_energy_tickets, has_migrated_tokens) =
                multi_arg.into_tuple();
            self.try_create_tickets(buyer.clone(), nr_staking_tickets + nr_energy_tickets);

            let mut user_ticket_status =
                UserTicketsStatus::new(nr_staking_tickets, nr_energy_tickets);

            if nr_staking_tickets >= min_confirmed_for_guaranteed_ticket {
                require!(
                    total_winning_tickets > 0,
                    "Too many users with guaranteed ticket"
                );
                let _ = guranteed_ticket_whitelist.insert(buyer.clone());
                total_winning_tickets -= STAKING_GUARANTEED_TICKETS_NO;
                total_guaranteed_tickets += STAKING_GUARANTEED_TICKETS_NO;
                user_ticket_status.staking_guaranteed_tickets = STAKING_GUARANTEED_TICKETS_NO;
            }

            if has_migrated_tokens {
                require!(
                    total_winning_tickets > 0,
                    "Too many users with guaranteed ticket"
                );
                let _ = guranteed_ticket_whitelist.insert(buyer.clone());
                total_winning_tickets -= MIGRATION_GUARANTEED_TICKETS_NO;
                total_guaranteed_tickets += MIGRATION_GUARANTEED_TICKETS_NO;
                user_ticket_status.migration_guaranteed_tickets = MIGRATION_GUARANTEED_TICKETS_NO;
            }

            self.user_ticket_status(&buyer).set(user_ticket_status);
        }

        self.total_guaranteed_tickets()
            .set(total_guaranteed_tickets);
        self.nr_winning_tickets().set(total_winning_tickets);
    }

    fn clear_users_with_guaranteed_ticket_after_blacklist(
        &self,
        users: &ManagedVec<ManagedAddress>,
    ) {
        let mut whitelist = self.users_with_guaranteed_ticket();
        let mut nr_winning_tickets_removed = 0;
        let mut total_guaranteed_tickets = self.total_guaranteed_tickets().get();
        for user in users {
            let was_whitelisted = whitelist.swap_remove(&user);
            if was_whitelisted {
                let user_ticket_status = self.user_ticket_status(&user).take();
                nr_winning_tickets_removed += user_ticket_status.staking_guaranteed_tickets;
                nr_winning_tickets_removed += user_ticket_status.migration_guaranteed_tickets;
                total_guaranteed_tickets -= user_ticket_status.staking_guaranteed_tickets;
                total_guaranteed_tickets -= user_ticket_status.migration_guaranteed_tickets;
            }
        }

        if nr_winning_tickets_removed > 0 {
            self.nr_winning_tickets()
                .update(|nr_winning| *nr_winning += nr_winning_tickets_removed);
        }
        self.total_guaranteed_tickets()
            .set(total_guaranteed_tickets);
    }

    #[storage_mapper("minConfirmedForGuaranteedTicket")]
    fn min_confirmed_for_guaranteed_ticket(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("usersWithGuaranteedTicket")]
    fn users_with_guaranteed_ticket(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("totalGuaranteedTickets")]
    fn total_guaranteed_tickets(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("userTicketStatus")]
    fn user_ticket_status(&self, user: &ManagedAddress) -> SingleValueMapper<UserTicketsStatus>;
}
