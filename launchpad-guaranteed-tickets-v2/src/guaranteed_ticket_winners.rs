multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use launchpad_common::{
    ongoing_operation::{CONTINUE_OP, STOP_OP},
    random::Random,
    tickets::{TicketRange, WINNING_TICKET},
};
use multiversx_sc::api::CryptoApi;

use crate::guaranteed_tickets_init::UserTicketsStatus;

const VEC_MAPPER_START_INDEX: usize = 1;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct GuaranteedTicketsSelectionOperation<M: ManagedTypeApi + CryptoApi> {
    pub rng: Random<M>,
    pub leftover_tickets: usize,
    pub leftover_ticket_pos_offset: usize,
    pub total_additional_winning_tickets: usize,
}

impl<M: ManagedTypeApi + CryptoApi> Default for GuaranteedTicketsSelectionOperation<M> {
    fn default() -> Self {
        Self {
            rng: Random::default(),
            leftover_tickets: 0,
            leftover_ticket_pos_offset: 1,
            total_additional_winning_tickets: 0,
        }
    }
}

pub struct CalculateGuaranteedTicketsResult {
    pub guaranteed_tickets: usize,
    pub leftover_tickets: usize,
}

impl CalculateGuaranteedTicketsResult {
    fn new(guaranteed_tickets: usize, leftover_tickets: usize) -> Self {
        Self {
            guaranteed_tickets,
            leftover_tickets,
        }
    }
}

pub enum AdditionalSelectionTryResult {
    Ok,
    CurrentAlreadyWinning,
    NewlySelectedAlreadyWinning,
}

#[multiversx_sc::module]
pub trait GuaranteedTicketWinnersModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::ongoing_operation::OngoingOperationModule
    + launchpad_common::tickets::TicketsModule
    + crate::guaranteed_tickets_init::GuaranteedTicketsInitModule
{
    fn select_guaranteed_tickets(
        &self,
        op: &mut GuaranteedTicketsSelectionOperation<Self::Api>,
    ) -> OperationCompletionStatus {
        let mut users_whitelist = self.users_with_guaranteed_ticket();
        let mut users_left = users_whitelist.len();

        self.run_while_it_has_gas(|| {
            if users_left == 0 {
                return STOP_OP;
            }

            let current_user = users_whitelist.get_by_index(VEC_MAPPER_START_INDEX);
            let _ = users_whitelist.swap_remove(&current_user);
            users_left -= 1;

            let user_ticket_status_mapper = self.user_ticket_status(&current_user);
            if user_ticket_status_mapper.is_empty() {
                return CONTINUE_OP;
            }
            let user_confirmed_tickets = self.nr_confirmed_tickets(&current_user).get();
            let user_ticket_status = user_ticket_status_mapper.get();

            let result =
                self.calculate_guaranteed_tickets(&user_ticket_status, user_confirmed_tickets);

            op.leftover_tickets += result.leftover_tickets;

            if result.guaranteed_tickets > 0 {
                self.process_guaranteed_tickets(&current_user, result.guaranteed_tickets, op);
            }

            CONTINUE_OP
        })
    }

    fn calculate_guaranteed_tickets(
        &self,
        user_ticket_status: &UserTicketsStatus<Self::Api>,
        user_confirmed_tickets: usize,
    ) -> CalculateGuaranteedTicketsResult {
        let mut guaranteed_tickets = 0;
        let mut leftover_tickets = 0;

        for info in user_ticket_status.guaranteed_tickets_info.iter() {
            if user_confirmed_tickets >= info.min_confirmed_tickets {
                guaranteed_tickets += info.guaranteed_tickets;
            } else {
                leftover_tickets += info.guaranteed_tickets;
            }
        }

        if guaranteed_tickets > user_confirmed_tickets {
            let excess_tickets = guaranteed_tickets - user_confirmed_tickets;
            leftover_tickets += excess_tickets;
            guaranteed_tickets = user_confirmed_tickets;
        }

        CalculateGuaranteedTicketsResult::new(guaranteed_tickets, leftover_tickets)
    }

    fn process_guaranteed_tickets(
        &self,
        user: &ManagedAddress,
        guaranteed_tickets: usize,
        op: &mut GuaranteedTicketsSelectionOperation<Self::Api>,
    ) {
        let ticket_range_mapper = self.ticket_range_for_address(user);
        if ticket_range_mapper.is_empty() {
            op.leftover_tickets += guaranteed_tickets;
            return;
        }
        let ticket_range = ticket_range_mapper.get();

        let user_winning_tickets = self.winning_tickets_in_range(&ticket_range);

        if guaranteed_tickets > user_winning_tickets {
            let tickets_to_win = guaranteed_tickets - user_winning_tickets;
            self.select_additional_winning_tickets(ticket_range, tickets_to_win, op);
        } else {
            op.leftover_tickets += guaranteed_tickets;
        }
    }

    fn select_additional_winning_tickets(
        &self,
        ticket_range: TicketRange,
        tickets_to_win: usize,
        op: &mut GuaranteedTicketsSelectionOperation<Self::Api>,
    ) {
        let mut remaining_tickets = tickets_to_win;
        let mut current_ticket = ticket_range.first_id;

        while remaining_tickets > 0 && current_ticket <= ticket_range.last_id {
            let is_winning_ticket = self.ticket_status(current_ticket).get();
            if !is_winning_ticket {
                self.ticket_status(current_ticket).set(WINNING_TICKET);
                op.total_additional_winning_tickets += 1;
                remaining_tickets -= 1;
            }
            current_ticket += 1;
        }

        op.leftover_tickets += remaining_tickets;
    }

    // TODO - add a check if current_ticket_pos > last_ticket_pos
    fn distribute_leftover_tickets(
        &self,
        op: &mut GuaranteedTicketsSelectionOperation<Self::Api>,
    ) -> OperationCompletionStatus {
        let nr_original_winning_tickets = self.nr_winning_tickets().get();
        let last_ticket_pos = self.get_total_tickets();

        self.run_while_it_has_gas(|| {
            if self.are_all_tickets_distributed(nr_original_winning_tickets, op, last_ticket_pos) {
                return STOP_OP;
            }

            self.distribute_single_leftover_ticket(op, nr_original_winning_tickets, last_ticket_pos)
        })
    }

    fn are_all_tickets_distributed(
        &self,
        nr_original_winning_tickets: usize,
        op: &mut GuaranteedTicketsSelectionOperation<Self::Api>,
        last_ticket_pos: usize,
    ) -> bool {
        if nr_original_winning_tickets + op.total_additional_winning_tickets >= last_ticket_pos {
            op.leftover_tickets = 0;
        }

        op.leftover_tickets == 0
    }

    fn distribute_single_leftover_ticket(
        &self,
        op: &mut GuaranteedTicketsSelectionOperation<Self::Api>,
        nr_original_winning_tickets: usize,
        last_ticket_pos: usize,
    ) -> bool {
        let current_ticket_pos = nr_original_winning_tickets + op.leftover_ticket_pos_offset;

        let selection_result = self.select_winning_ticket(op, current_ticket_pos, last_ticket_pos);

        self.process_selection_result(op, selection_result)
    }

    fn select_winning_ticket(
        &self,
        op: &mut GuaranteedTicketsSelectionOperation<Self::Api>,
        current_ticket_pos: usize,
        last_ticket_pos: usize,
    ) -> AdditionalSelectionTryResult {
        self.try_select_winning_ticket(&mut op.rng, current_ticket_pos, last_ticket_pos)
    }

    fn process_selection_result(
        &self,
        op: &mut GuaranteedTicketsSelectionOperation<Self::Api>,
        selection_result: AdditionalSelectionTryResult,
    ) -> bool {
        match selection_result {
            AdditionalSelectionTryResult::Ok => {
                op.leftover_tickets -= 1;
                op.total_additional_winning_tickets += 1;
                op.leftover_ticket_pos_offset += 1;
            }
            AdditionalSelectionTryResult::CurrentAlreadyWinning
            | AdditionalSelectionTryResult::NewlySelectedAlreadyWinning => {
                op.leftover_ticket_pos_offset += 1;
            }
        }

        CONTINUE_OP
    }

    fn winning_tickets_in_range(&self, ticket_range: &TicketRange) -> usize {
        let mut winning_tickets_no = 0;
        for ticket_id in ticket_range.first_id..=ticket_range.last_id {
            let ticket_status = self.ticket_status(ticket_id).get();
            if ticket_status == WINNING_TICKET {
                winning_tickets_no += 1;
            }
        }

        winning_tickets_no
    }

    fn try_select_winning_ticket(
        &self,
        rng: &mut Random<Self::Api>,
        current_ticket_position: usize,
        last_ticket_position: usize,
    ) -> AdditionalSelectionTryResult {
        let current_ticket_id = self.get_ticket_id_from_pos(current_ticket_position);
        if self.is_already_winning_ticket(current_ticket_id) {
            return AdditionalSelectionTryResult::CurrentAlreadyWinning;
        }

        let rand_pos = rng.next_usize_in_range(current_ticket_position, last_ticket_position + 1);
        let winning_ticket_id = self.get_ticket_id_from_pos(rand_pos);
        if self.is_already_winning_ticket(winning_ticket_id) {
            return AdditionalSelectionTryResult::NewlySelectedAlreadyWinning;
        }

        self.ticket_pos_to_id(rand_pos).set(current_ticket_id);
        self.ticket_status(winning_ticket_id).set(WINNING_TICKET);

        AdditionalSelectionTryResult::Ok
    }

    #[inline]
    fn is_already_winning_ticket(&self, ticket_id: usize) -> bool {
        self.ticket_status(ticket_id).get() == WINNING_TICKET
    }
}
