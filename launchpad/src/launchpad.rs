#![no_std]

elrond_wasm::imports!();
use elrond_wasm::elrond_codec::TopEncode;

mod setup;

mod ongoing_operation;
use ongoing_operation::*;

mod random;
use random::Random;

mod ticket_status;
use ticket_status::TicketStatus;

const VEC_MAPPER_START_INDEX: usize = 1;

#[elrond_wasm::derive::contract]
pub trait Launchpad: setup::SetupModule + ongoing_operation::OngoingOperationModule {
    // endpoints - owner-only

    #[only_owner]
    #[endpoint]
    fn add_tickets(
        &self,
        #[var_args] address_number_pairs: VarArgs<MultiArg2<Address, usize>>,
    ) -> SCResult<OperationCompletionStatus> {
        let ongoing_operation = self.current_ongoing_operation().get();
        let mut index = match ongoing_operation {
            OngoingOperationType::None => 0,
            OngoingOperationType::AddTickets { index } => {
                self.clear_operation();
                index
            }
            _ => return sc_error!("Another ongoing operation is in progress"),
        };

        let address_number_pairs_vec = address_number_pairs.into_vec();
        let nr_pairs = address_number_pairs_vec.len();

        let gas_before = self.blockchain().get_gas_left();

        let (first_buyer, first_nr_tickets) = address_number_pairs_vec[index].clone().into_tuple();
        self.create_tickets(&first_buyer, first_nr_tickets);
        index += 1;

        let gas_after = self.blockchain().get_gas_left();
        let gas_per_iteration = (gas_before - gas_after) / first_nr_tickets as u64;

        while index < nr_pairs {
            let (buyer, nr_tickets) = address_number_pairs_vec[index].clone().into_tuple();
            let gas_cost = gas_per_iteration * nr_tickets as u64;

            if self.can_continue_operation(gas_cost) {
                self.create_tickets(&buyer, nr_tickets);
                index += 1;
            } else {
                self.save_progress(&OngoingOperationType::AddTickets { index });

                return Ok(OperationCompletionStatus::InterruptedBeforeOutOfGas);
            }
        }

        // signal the start of claiming period
        let current_epoch = self.blockchain().get_block_epoch();
        self.claim_period_start_epoch().set(&current_epoch);

        Ok(OperationCompletionStatus::Completed)
    }

    // endpoints

    #[endpoint(selectWinners)]
    fn select_winners(&self) -> SCResult<OperationCompletionStatus> {
        require!(
            self.confirmation_period_start_epoch().is_empty(),
            "Cannot select winners after confirmation period started"
        );

        let current_epoch = self.blockchain().get_block_epoch();
        let winner_selection_start_epoch = self.winner_selection_start_epoch().get();

        require!(
            winner_selection_start_epoch >= current_epoch,
            "Cannot select winners yet"
        );

        let ongoing_operation = self.current_ongoing_operation().get();
        let (mut rng, mut ticket_position) = match ongoing_operation {
            OngoingOperationType::None => (
                Random::from_seeds(
                    self.crypto(),
                    self.blockchain().get_prev_block_random_seed(),
                    self.blockchain().get_block_random_seed(),
                ),
                VEC_MAPPER_START_INDEX,
            ),
            OngoingOperationType::SelectWinners {
                seed,
                seed_index,
                ticket_position,
            } => {
                self.clear_operation();

                (
                    Random::from_hash(self.crypto(), seed, seed_index),
                    ticket_position,
                )
            }
            _ => return sc_error!("Another ongoing operation is in progress"),
        };

        let last_ticket_position = self.ticket_owners().len();

        let gas_before = self.blockchain().get_gas_left();

        self.select_single_winning_ticket(&mut rng, ticket_position, last_ticket_position);
        ticket_position += 1;

        let gas_after = self.blockchain().get_gas_left();
        let gas_per_iteration = gas_before - gas_after;

        while ticket_position < last_ticket_position - 1 {
            if self.can_continue_operation(gas_per_iteration) {
                self.select_single_winning_ticket(&mut rng, ticket_position, last_ticket_position);
                ticket_position += 1;
            } else {
                self.save_progress(&OngoingOperationType::SelectWinners {
                    seed: rng.seed,
                    seed_index: rng.index,
                    ticket_position,
                });

                return Ok(OperationCompletionStatus::InterruptedBeforeOutOfGas);
            }
        }

        self.start_confirmation_period();

        Ok(OperationCompletionStatus::Completed)
    }

    #[payable("*")]
    #[endpoint(confirmTickets)]
    fn confirm_tickets(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: Self::BigUint,
        nr_tickets_to_confirm: usize,
    ) -> SCResult<()> {
        self.require_no_ongoing_operation()?;

        let current_epoch = self.blockchain().get_block_epoch();
        let confirmation_start_epoch = self.confirmation_period_start_epoch().get();
        let confirmation_period_in_epochs = self.confirmation_period_in_epochs().get();
        let confirmation_end_epoch = confirmation_start_epoch + confirmation_period_in_epochs;

        require!(
            current_epoch > confirmation_start_epoch,
            "Confirmation has not started yet"
        );
        require!(
            current_epoch <= confirmation_end_epoch,
            "Confirmation period has ended"
        );

        let ticket_payment_token = self.ticket_payment_token().get();
        let ticket_price = self.ticket_price().get();
        let total_ticket_price = Self::BigUint::from(nr_tickets_to_confirm) * ticket_price;

        require!(
            payment_token == ticket_payment_token,
            "Wrong payment token used"
        );
        require!(payment_amount == total_ticket_price, "Wrong amount sent");

        let caller = self.blockchain().get_caller();
        require!(
            !self.ticket_range_for_address(&caller).is_empty(),
            "You have no tickets"
        );

        let (first_ticket_id, last_ticket_id) = self.ticket_range_for_address(&caller).get();
        let nr_tickets = last_ticket_id - first_ticket_id + 1;

        require!(
            nr_tickets >= nr_tickets_to_confirm,
            "Trying to confirm too many tickets"
        );

        let mut actual_confirmed_tickets = 0;
        for ticket_id in first_ticket_id..=last_ticket_id {
            let ticket_status = self.ticket_status().get(ticket_id);
            if ticket_status != TicketStatus::Winning {
                continue;
            }

            self.set_ticket_status(ticket_id, TicketStatus::Confirmed);
            actual_confirmed_tickets += 1;

            if actual_confirmed_tickets == nr_tickets_to_confirm {
                break;
            }
        }

        require!(
            actual_confirmed_tickets == nr_tickets_to_confirm,
            "Couldn't confirm all tickets"
        );

        Ok(())
    }

    // views

    #[view(getNumberOfWinningTicketsForAddress)]
    fn get_number_of_winning_tickets_for_address(&self, address: Address) -> usize {
        if self.ticket_range_for_address(&address).is_empty() {
            return 0;
        }

        let mut nr_winning_tickets = 0;
        let (first_ticket_id, last_ticket_id) = self.ticket_range_for_address(&address).get();

        for ticket_id in first_ticket_id..=last_ticket_id {
            let ticket_status = self.ticket_status().get(ticket_id);
            if ticket_status != TicketStatus::Winning {
                nr_winning_tickets += 1;
            }
        }

        nr_winning_tickets
    }
    
    // private

    fn create_tickets(&self, buyer: &Address, nr_tickets: usize) {
        let first_ticket_id = self.ticket_owners().len() + 1;
        let last_ticket_id = first_ticket_id + nr_tickets - 1;
        self.ticket_range_for_address(buyer)
            .set(&(first_ticket_id, last_ticket_id));

        for _ in 0..nr_tickets {
            let ticket_id = self.ticket_owners().push(buyer);
            self.ticket_status().push(&TicketStatus::Normal);
            self.shuffled_tickets().push(&ticket_id);
        }
    }

    /// Fisher-Yates algorithm,
    /// each position is swapped with a random one that's after it,
    /// as we select the first N positions as winning, once a position has been swapped,
    /// it's guaranteed to not be swapped again, so we can mark the ticket as Winning
    fn select_single_winning_ticket(
        &self,
        rng: &mut Random<Self::CryptoApi>,
        current_ticket_position: usize,
        last_ticket_position: usize,
    ) {
        let rand_index =
            rng.next_usize_in_range(current_ticket_position + 1, last_ticket_position + 1);
        self.swap(self.shuffled_tickets(), current_ticket_position, rand_index);

        let winning_ticket_id = self.shuffled_tickets().get(current_ticket_position);
        self.set_ticket_status(winning_ticket_id, TicketStatus::Winning);
    }

    fn swap<T: TopEncode + TopDecode>(
        &self,
        mapper: VecMapper<Self::Storage, T>,
        first_index: usize,
        second_index: usize,
    ) {
        let first_element = mapper.get(first_index);
        let second_element = mapper.get(second_index);

        mapper.set(first_index, &second_element);
        mapper.set(second_index, &first_element);
    }

    fn set_ticket_status(&self, ticket_id: usize, status: TicketStatus) {
        self.ticket_status().set(ticket_id, &status);
    }

    fn start_confirmation_period(&self) {
        let current_epoch = self.blockchain().get_block_epoch();
        self.confirmation_period_start_epoch().set(&current_epoch);
    }

    // storage

    // ticket ID -> address mapping
    #[storage_mapper("ticketOwners")]
    fn ticket_owners(&self) -> VecMapper<Self::Storage, Address>;

    #[storage_mapper("ticketStatus")]
    fn ticket_status(&self) -> VecMapper<Self::Storage, TicketStatus>;

    #[storage_mapper("ticketRangeForAddress")]
    fn ticket_range_for_address(
        &self,
        address: &Address,
    ) -> SingleValueMapper<Self::Storage, (usize, usize)>;

    #[storage_mapper("shuffledTickets")]
    fn shuffled_tickets(&self) -> VecMapper<Self::Storage, usize>;

    #[view(getConfirmationPeriodStartEpoch)]
    #[storage_mapper("confirmationPeriodStartEpoch")]
    fn confirmation_period_start_epoch(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[view(getClaimPeriodStartEpoch)]
    #[storage_mapper("claimPeriodStartEpoch")]
    fn claim_period_start_epoch(&self) -> SingleValueMapper<Self::Storage, u64>;
}
