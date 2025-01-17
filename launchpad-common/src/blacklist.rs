multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait BlacklistModule:
    crate::permissions::PermissionsModule
    + crate::launch_stage::LaunchStageModule
    + crate::tickets::TicketsModule
    + crate::token_send::TokenSendModule
    + crate::config::ConfigModule
    + crate::common_events::CommonEventsModule
{
    fn add_users_to_blacklist(&self, users_list: &ManagedVec<ManagedAddress>) {
        self.require_extended_permissions();
        self.add_users_to_blacklist_without_permissions(users_list);
    }

    fn add_users_to_blacklist_without_permissions(&self, users_list: &ManagedVec<ManagedAddress>) {
        self.require_before_winner_selection();

        let blacklist_mapper = self.blacklist();
        for address in users_list {
            require!(
                !blacklist_mapper.contains(&address),
                "User already blacklisted"
            );
            let confirmed_tickets_mapper = self.nr_confirmed_tickets(&address);
            let nr_confirmed_tickets = confirmed_tickets_mapper.get();
            if nr_confirmed_tickets > 0 {
                self.refund_ticket_payment(&address, nr_confirmed_tickets);
                confirmed_tickets_mapper.clear();
            }

            blacklist_mapper.add(&address);
        }
    }

    fn remove_users_from_blacklist(&self, users_list: MultiValueEncoded<ManagedAddress>) {
        self.require_extended_permissions();
        self.require_before_winner_selection();

        let blacklist_mapper = self.blacklist();
        for address in users_list {
            require!(
                blacklist_mapper.contains(&address),
                "User is not blacklisted"
            );
            blacklist_mapper.remove(&address);
        }
    }

    #[view(isUserBlacklisted)]
    fn is_user_blacklisted(&self, address: &ManagedAddress) -> bool {
        self.blacklist().contains(address)
    }

    #[storage_mapper("blacklisted")]
    fn blacklist(&self) -> WhitelistMapper<Self::Api, ManagedAddress>;
}
