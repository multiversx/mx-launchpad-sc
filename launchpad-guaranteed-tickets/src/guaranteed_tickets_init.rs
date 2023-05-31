multiversx_sc::imports!();

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

                let _ = guranteed_ticket_whitelist.insert(buyer);
                total_winning_tickets -= 1;
            }
        }

        self.nr_winning_tickets().set(total_winning_tickets);
    }

    fn clear_users_with_guaranteed_ticket_after_blacklist(
        &self,
        users: &ManagedVec<ManagedAddress>,
    ) {
        let mut whitelist = self.users_with_guaranteed_ticket();
        let mut nr_users_removed = 0;
        for user in users {
            let was_whitelisted = whitelist.swap_remove(&user);
            if was_whitelisted {
                nr_users_removed += 1;
            }
        }

        if nr_users_removed > 0 {
            self.nr_winning_tickets()
                .update(|nr_winning| *nr_winning += nr_users_removed);
        }
    }

    #[storage_mapper("minConfirmedForGuaranteedTicket")]
    fn min_confirmed_for_guaranteed_ticket(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("usersWithGuaranteedTicket")]
    fn users_with_guaranteed_ticket(&self) -> UnorderedSetMapper<ManagedAddress>;
}
