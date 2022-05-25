elrond_wasm::imports!();

use crate::{
    config::TokenAmountPair,
    launch_stage::Flags,
    ongoing_operation::{OngoingOperationType, CONTINUE_OP, STOP_OP},
    random::Random,
    tickets::{TicketBatch, TicketRange, FIRST_TICKET_ID, WINNING_TICKET},
};

#[elrond_wasm::module]
pub trait WinnerSelectionModule:
    crate::launch_stage::LaunchStageModule
    + crate::tickets::TicketsModule
    + crate::ongoing_operation::OngoingOperationModule
    + crate::config::ConfigModule
    + crate::blacklist::BlacklistModule
    + crate::token_send::TokenSendModule
    + crate::permissions::PermissionsModule
{
    #[endpoint(filterTickets)]
    fn filter_tickets(&self) -> OperationCompletionStatus {
        self.require_winner_selection_period();

        let flags_mapper = self.flags();
        let mut flags: Flags = flags_mapper.get();
        require!(!flags.were_tickets_filtered, "Tickets already filtered");

        let last_ticket_id = self.last_ticket_id().get();
        let (mut first_ticket_id_in_batch, mut nr_removed) = self.load_filter_tickets_operation();

        if first_ticket_id_in_batch == FIRST_TICKET_ID {
            flags.has_winner_selection_process_started = true;
        }

        let run_result = self.run_while_it_has_gas(|| {
            let current_ticket_batch_mapper = self.ticket_batch(first_ticket_id_in_batch);
            let ticket_batch: TicketBatch<Self::Api> = current_ticket_batch_mapper.get();
            let address = &ticket_batch.address;
            let nr_tickets_in_batch = ticket_batch.nr_tickets;

            let nr_confirmed_tickets = self.nr_confirmed_tickets(address).get();
            if self.is_user_blacklisted(address) || nr_confirmed_tickets == 0 {
                self.ticket_range_for_address(address).clear();
                current_ticket_batch_mapper.clear();
            } else if nr_removed > 0 || nr_confirmed_tickets < nr_tickets_in_batch {
                let new_first_id = first_ticket_id_in_batch - nr_removed;
                let new_last_id = new_first_id + nr_confirmed_tickets - 1;

                current_ticket_batch_mapper.clear();

                self.ticket_range_for_address(address).set(&TicketRange {
                    first_id: new_first_id,
                    last_id: new_last_id,
                });
                self.ticket_batch(new_first_id).set(&TicketBatch {
                    address: ticket_batch.address,
                    nr_tickets: nr_confirmed_tickets,
                });
            }

            nr_removed += nr_tickets_in_batch - nr_confirmed_tickets;
            first_ticket_id_in_batch += nr_tickets_in_batch;

            if first_ticket_id_in_batch == last_ticket_id + 1 {
                STOP_OP
            } else {
                CONTINUE_OP
            }
        });

        match run_result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                self.save_progress(&OngoingOperationType::FilterTickets {
                    first_ticket_id_in_batch,
                    nr_removed,
                });
            }
            OperationCompletionStatus::Completed => {
                // this only happens when a lot of tickets have been eliminated,
                // and we end up with less total tickets than winning
                let new_last_ticket_id = last_ticket_id - nr_removed;
                let nr_winning_tickets = self.nr_winning_tickets().get();
                if nr_winning_tickets > new_last_ticket_id {
                    self.nr_winning_tickets().set(&new_last_ticket_id);
                }

                self.last_ticket_id().set(&new_last_ticket_id);
                flags.were_tickets_filtered = true;
            }
        };

        flags_mapper.set(&flags);

        run_result
    }

    #[endpoint(selectWinners)]
    fn select_winners(&self) -> OperationCompletionStatus {
        self.require_winner_selection_period();

        let flags_mapper = self.flags();
        let mut flags: Flags = flags_mapper.get();
        require!(flags.were_tickets_filtered, "Must filter tickets first");
        require!(!flags.were_winners_selected, "Winners already selected");

        let nr_winning_tickets = self.nr_winning_tickets().get();
        let last_ticket_position = self.get_total_tickets();

        let (mut rng, mut ticket_position) = self.load_select_winners_operation();
        let run_result = self.run_while_it_has_gas(|| {
            self.shuffle_single_ticket(&mut rng, ticket_position, last_ticket_position);

            if ticket_position == nr_winning_tickets {
                return STOP_OP;
            }

            ticket_position += 1;

            CONTINUE_OP
        });

        match run_result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                self.save_progress(&OngoingOperationType::SelectWinners {
                    rng,
                    ticket_position,
                });
            }
            OperationCompletionStatus::Completed => {
                flags.were_winners_selected = true;

                let ticket_price: TokenAmountPair<Self::Api> = self.ticket_price().get();
                let claimable_ticket_payment = ticket_price.amount * (nr_winning_tickets as u32);
                self.claimable_ticket_payment()
                    .set(&claimable_ticket_payment);
            }
        };

        flags_mapper.set(&flags);

        run_result
    }

    /// Fisher-Yates algorithm,
    /// each position i is swapped with a random one in range [i, n]
    fn shuffle_single_ticket(
        &self,
        rng: &mut Random<Self::Api>,
        current_ticket_position: usize,
        last_ticket_position: usize,
    ) {
        let rand_pos = rng.next_usize_in_range(current_ticket_position, last_ticket_position + 1);

        let winning_ticket_id = self.get_ticket_id_from_pos(rand_pos);
        self.ticket_status(winning_ticket_id).set(WINNING_TICKET);

        let current_ticket_id = self.get_ticket_id_from_pos(current_ticket_position);
        self.ticket_pos_to_id(rand_pos).set(current_ticket_id);
    }

    #[view(getNumberOfWinningTicketsForAddress)]
    fn get_number_of_winning_tickets_for_address(&self, address: ManagedAddress) -> usize {
        self.get_winning_ticket_ids_for_address(address).len()
    }

    #[view(getWinningTicketIdsForAddress)]
    fn get_winning_ticket_ids_for_address(
        &self,
        address: ManagedAddress,
    ) -> MultiValueEncoded<usize> {
        let flags: Flags = self.flags().get();
        let ticket_range_mapper = self.ticket_range_for_address(&address);
        let mut ticket_ids = MultiValueEncoded::new();
        if !flags.were_winners_selected || ticket_range_mapper.is_empty() {
            return ticket_ids;
        }

        let ticket_range: TicketRange = ticket_range_mapper.get();
        for ticket_id in ticket_range.first_id..=ticket_range.last_id {
            let actual_ticket_status = self.ticket_status(ticket_id).get();
            if actual_ticket_status == WINNING_TICKET {
                ticket_ids.push(ticket_id);
            }
        }

        ticket_ids
    }
}
