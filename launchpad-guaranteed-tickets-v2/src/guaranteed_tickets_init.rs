multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct GuaranteedTicketInfo {
    pub guaranteed_tickets: usize,
    pub min_confirmed_tickets: usize,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct UserTicketsStatus<M: ManagedTypeApi> {
    pub total_tickets_allowance: usize,
    pub guaranteed_tickets_info: ManagedVec<M, GuaranteedTicketInfo>,
}

impl<M: ManagedTypeApi> UserTicketsStatus<M> {
    fn new(total_tickets_allowance: usize) -> Self {
        Self {
            total_tickets_allowance,
            guaranteed_tickets_info: ManagedVec::new(),
        }
    }
}

pub struct AddTicketsResult {
    pub total_tickets_added: usize,
    pub total_guaranteed_tickets_added: usize,
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
        address_number_pairs: MultiValueEncoded<
            MultiValue3<ManagedAddress, usize, ManagedVec<GuaranteedTicketInfo>>,
        >,
    ) -> AddTicketsResult {
        self.require_add_tickets_period();

        let mut guaranteed_ticket_whitelist = self.users_with_guaranteed_ticket();
        let mut total_winning_tickets = self.nr_winning_tickets().get();
        let mut total_guaranteed_tickets = self.total_guaranteed_tickets().get();

        let mut total_tickets_added = 0;
        let mut total_guaranteed_tickets_added = 0;

        for multi_arg in address_number_pairs {
            let (buyer, total_tickets_allowance, guaranteed_ticket_infos) = multi_arg.into_tuple();
            self.try_create_tickets(buyer.clone(), total_tickets_allowance);

            let mut user_ticket_status = UserTicketsStatus::new(total_tickets_allowance);

            let mut user_guaranteed_tickets = 0;
            for info in guaranteed_ticket_infos.iter() {
                require!(
                    info.guaranteed_tickets <= info.min_confirmed_tickets,
                    "Invalid guaranteed ticket min confirmed tickets"
                );
                user_guaranteed_tickets += info.guaranteed_tickets;
            }

            if user_guaranteed_tickets > 0 {
                require!(
                    total_winning_tickets >= user_guaranteed_tickets,
                    "Not enough winning tickets for guaranteed allocation"
                );
                let _ = guaranteed_ticket_whitelist.insert(buyer.clone());
                total_winning_tickets -= user_guaranteed_tickets;
                total_guaranteed_tickets += user_guaranteed_tickets;
                user_ticket_status.guaranteed_tickets_info = guaranteed_ticket_infos;
                total_guaranteed_tickets_added += user_guaranteed_tickets;
            }
            total_tickets_added += total_tickets_allowance;

            self.user_ticket_status(&buyer).set(user_ticket_status);
        }

        self.total_guaranteed_tickets()
            .set(total_guaranteed_tickets);
        self.nr_winning_tickets().set(total_winning_tickets);

        AddTicketsResult {
            total_tickets_added,
            total_guaranteed_tickets_added,
        }
    }

    fn clear_users_with_guaranteed_ticket_after_blacklist(
        &self,
        users: &ManagedVec<ManagedAddress>,
    ) {
        let mut whitelist = self.users_with_guaranteed_ticket();
        let mut nr_winning_tickets = self.nr_winning_tickets().get();
        let mut total_guaranteed_tickets = self.total_guaranteed_tickets().get();
        for user in users {
            let was_whitelisted = whitelist.swap_remove(&user);
            if was_whitelisted {
                let user_ticket_status = self.user_ticket_status(&user).take();
                let guaranteed_tickets_recovered = user_ticket_status
                    .guaranteed_tickets_info
                    .iter()
                    .fold(0, |acc, info| acc + info.guaranteed_tickets);

                nr_winning_tickets += guaranteed_tickets_recovered;
                total_guaranteed_tickets -= guaranteed_tickets_recovered;
                self.blacklist_user_ticket_status(&user)
                    .set(user_ticket_status);
            }
        }

        self.nr_winning_tickets().set(nr_winning_tickets);
        self.total_guaranteed_tickets()
            .set(total_guaranteed_tickets);
    }

    fn remove_guaranteed_tickets_from_blacklist(&self, users: &ManagedVec<ManagedAddress>) {
        let mut nr_winning_tickets = self.nr_winning_tickets().get();
        let mut total_guaranteed_tickets = self.total_guaranteed_tickets().get();
        let mut whitelist = self.users_with_guaranteed_ticket();
        for user in users {
            let user_ticket_status_mapper = self.user_ticket_status(&user);
            if !user_ticket_status_mapper.is_empty()
                || self.ticket_range_for_address(&user).is_empty()
            {
                continue;
            }

            let user_ticket_status = self.blacklist_user_ticket_status(&user).take();
            let guaranteed_tickets_added = user_ticket_status
                .guaranteed_tickets_info
                .iter()
                .fold(0, |acc, info| acc + info.guaranteed_tickets);

            if guaranteed_tickets_added > 0 {
                whitelist.insert(user.clone());
                nr_winning_tickets -= guaranteed_tickets_added;
                total_guaranteed_tickets += guaranteed_tickets_added;
                user_ticket_status_mapper.set(user_ticket_status);
            }
        }

        self.nr_winning_tickets().set(nr_winning_tickets);
        self.total_guaranteed_tickets()
            .set(total_guaranteed_tickets);
    }

    #[storage_mapper("usersWithGuaranteedTicket")]
    fn users_with_guaranteed_ticket(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("totalGuaranteedTickets")]
    fn total_guaranteed_tickets(&self) -> SingleValueMapper<usize>;
    #[storage_mapper("userTicketStatus")]
    fn user_ticket_status(
        &self,
        user: &ManagedAddress,
    ) -> SingleValueMapper<UserTicketsStatus<Self::Api>>;

    #[storage_mapper("blacklistUserTicketStatus")]
    fn blacklist_user_ticket_status(
        &self,
        user: &ManagedAddress,
    ) -> SingleValueMapper<UserTicketsStatus<Self::Api>>;
}
