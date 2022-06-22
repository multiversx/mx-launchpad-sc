elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait GuaranteedTicketsInitModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::ongoing_operation::OngoingOperationModule
    + launchpad_common::tickets::TicketsModule
    + crate::guranteed_ticket_winners::GuaranteedTicketWinnersModule
{
    fn add_tickets_with_guaranteed_winners(
        &self,
        address_number_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, usize>>,
    ) {
        self.require_add_tickets_period();

        let max_tier_tickets = self.max_tier_tickets().get();
        let mut max_tier_whitelist = self.max_tier_users();
        let mut total_winning_tickets = self.nr_winning_tickets().get();

        for multi_arg in address_number_pairs {
            let (buyer, nr_tickets) = multi_arg.into_tuple();
            require!(nr_tickets <= max_tier_tickets, "Too many tickets for user");

            self.try_create_tickets(buyer.clone(), nr_tickets);

            if nr_tickets == max_tier_tickets {
                require!(total_winning_tickets > 0, "Too many max tier users");

                let _ = max_tier_whitelist.insert(buyer);
                total_winning_tickets -= 1;
            }
        }

        self.nr_winning_tickets().set(total_winning_tickets);
    }

    fn clear_max_tier_users_after_blacklist(&self, users: &ManagedVec<ManagedAddress>) {
        let mut max_tier_whitelist = self.max_tier_users();
        let mut nr_max_tier_removed = 0;
        for user in users {
            let was_whitelisted = max_tier_whitelist.swap_remove(&user);
            if was_whitelisted {
                nr_max_tier_removed += 1;
            }
        }

        if nr_max_tier_removed > 0 {
            self.nr_winning_tickets()
                .update(|nr_winning| *nr_winning += nr_max_tier_removed);
        }
    }
}
