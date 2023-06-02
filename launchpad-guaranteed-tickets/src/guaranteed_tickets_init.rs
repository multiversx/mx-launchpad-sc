multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const STAKING_GUARANTEED_TICKETS_NO: usize = 1;
pub const MIGRATION_GUARANTEED_TICKETS_NO: usize = 1;

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone)]
pub struct UserGuaranteedTickets<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub guaranteed_tickets: usize,
}

impl<M: ManagedTypeApi> UserGuaranteedTickets<M> {
    pub fn new(address: ManagedAddress<M>, guaranteed_tickets: usize) -> Self {
        Self {
            address,
            guaranteed_tickets,
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
        address_number_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) {
        self.require_add_tickets_period();

        let min_confirmed_for_guaranteed_ticket = self.min_confirmed_for_guaranteed_ticket().get();
        let mut guranteed_ticket_whitelist = self.users_with_guaranteed_ticket();
        let mut total_winning_tickets = self.nr_winning_tickets().get();

        for multi_arg in address_number_pairs {
            let (buyer, nr_tickets) = multi_arg.into_tuple();
            self.try_create_tickets(buyer.clone(), nr_tickets);

            if nr_tickets >= min_confirmed_for_guaranteed_ticket {
                require!(
                    total_winning_tickets > 0,
                    "Too many users with guaranteed ticket"
                );

                let user_guaranteed_tickets =
                    UserGuaranteedTickets::new(buyer, STAKING_GUARANTEED_TICKETS_NO);
                let _ = guranteed_ticket_whitelist.insert(user_guaranteed_tickets);
                total_winning_tickets -= STAKING_GUARANTEED_TICKETS_NO;
            }
        }

        self.nr_winning_tickets().set(total_winning_tickets);
    }

    fn add_more_guaranteed_tickets(&self, addresses: MultiValueEncoded<ManagedAddress>) {
        self.require_add_tickets_period();

        let mut guranteed_ticket_whitelist = self.users_with_guaranteed_ticket();
        let mut total_winning_tickets = self.nr_winning_tickets().get();

        for user in addresses {
            let mut user_new_guaranteed_tickets = MIGRATION_GUARANTEED_TICKETS_NO;
            let user_initial_guaranteed_tickets =
                UserGuaranteedTickets::new(user.clone(), STAKING_GUARANTEED_TICKETS_NO);
            if guranteed_ticket_whitelist.swap_remove(&user_initial_guaranteed_tickets) {
                user_new_guaranteed_tickets += 1;
            }
            let user_ticket_range = self.ticket_range_for_address(&user).get();
            let user_total_tickets_no = user_ticket_range.last_id - user_ticket_range.first_id + 1;

            require!(
                total_winning_tickets > 0,
                "Too many users with guaranteed ticket"
            );
            require!(
                user_total_tickets_no >= user_new_guaranteed_tickets,
                "The guaranteed tickets number is bigger than the user's total tickets"
            );

            let new_user_guaranteed_tickets =
                UserGuaranteedTickets::new(user, user_new_guaranteed_tickets);

            let _ = guranteed_ticket_whitelist.insert(new_user_guaranteed_tickets);
            total_winning_tickets -= STAKING_GUARANTEED_TICKETS_NO;
        }

        self.nr_winning_tickets().set(total_winning_tickets);
    }

    fn clear_users_with_guaranteed_ticket_after_blacklist(
        &self,
        users: &ManagedVec<ManagedAddress>,
    ) {
        let mut whitelist = self.users_with_guaranteed_ticket();
        let mut nr_tickets_removed = 0;
        for user in users {
            let user_staking_guaranteed_tickets =
                UserGuaranteedTickets::new(user.clone(), STAKING_GUARANTEED_TICKETS_NO);
            let user_migration_guaranteed_tickets = UserGuaranteedTickets::new(
                user,
                STAKING_GUARANTEED_TICKETS_NO + MIGRATION_GUARANTEED_TICKETS_NO,
            );
            if whitelist.swap_remove(&user_staking_guaranteed_tickets) {
                nr_tickets_removed += user_staking_guaranteed_tickets.guaranteed_tickets;
            }
            if whitelist.swap_remove(&user_migration_guaranteed_tickets) {
                nr_tickets_removed += user_migration_guaranteed_tickets.guaranteed_tickets;
            }
        }

        if nr_tickets_removed > 0 {
            self.nr_winning_tickets()
                .update(|nr_winning| *nr_winning += nr_tickets_removed);
        }
    }

    #[storage_mapper("minConfirmedForGuaranteedTicket")]
    fn min_confirmed_for_guaranteed_ticket(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("usersWithGuaranteedTicket")]
    fn users_with_guaranteed_ticket(&self) -> UnorderedSetMapper<UserGuaranteedTickets<Self::Api>>;
}
