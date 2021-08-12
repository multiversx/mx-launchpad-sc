#![no_std]

elrond_wasm::imports!();
use elrond_wasm::elrond_codec::TopEncode;

mod setup;

mod ongoing_operation;
use ongoing_operation::*;

mod random;
use random::Random;

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

        Ok(OperationCompletionStatus::Completed)
    }

    // endpoints

    #[endpoint(selectWinners)]
    fn select_winners(&self) -> SCResult<OperationCompletionStatus> {
        require!(
            !self.winner_selection_start_epoch().is_empty(),
            "Winner selection start epoch not set"
        );
        require!(
            self.claim_period_start_epoch().is_empty(),
            "Cannot select winners after claim period started"
        );

        let current_epoch = self.blockchain().get_block_epoch();
        let winner_selection_start_epoch = self.winner_selection_start_epoch().get();

        require!(
            winner_selection_start_epoch >= current_epoch,
            "Cannot select winners yet"
        );

        let ongoing_operation = self.current_ongoing_operation().get();
        let (mut rng, mut ticket_index) = match ongoing_operation {
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
                ticket_index,
            } => {
                self.clear_operation();

                (
                    Random::from_hash(self.crypto(), seed, seed_index),
                    ticket_index,
                )
            }
            _ => return sc_error!("Another ongoing operation is in progress"),
        };

        let last_ticket_id = self.ticket_owners().len();

        let gas_before = self.blockchain().get_gas_left();

        let mut rand_index = rng.next_usize_in_range(ticket_index + 1, last_ticket_id);
        self.swap(self.shuffled_tickets(), ticket_index, rand_index);
        ticket_index += 1;

        let gas_after = self.blockchain().get_gas_left();
        let gas_per_iteration = gas_before - gas_after;

        while ticket_index < last_ticket_id - 1 {
            if self.can_continue_operation(gas_per_iteration) {
                rand_index = rng.next_usize_in_range(ticket_index + 1, last_ticket_id);
                self.swap(self.shuffled_tickets(), ticket_index, rand_index);
                ticket_index += 1;
            } else {
                self.save_progress(&OngoingOperationType::SelectWinners {
                    seed: rng.seed,
                    seed_index: rng.index,
                    ticket_index,
                });

                return Ok(OperationCompletionStatus::InterruptedBeforeOutOfGas);
            }
        }

        Ok(OperationCompletionStatus::Completed)
    }

    // private

    fn create_tickets(&self, buyer: &Address, nr_tickets: usize) {
        for _ in 0..nr_tickets {
            let ticket_id = self.ticket_owners().push(buyer);
            self.shuffled_tickets().push(&ticket_id);
            self.uncalimed_tickets_for_address(buyer).insert(ticket_id);
        }
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

    // storage

    // ticket ID -> address mapping
    #[storage_mapper("ticketOwners")]
    fn ticket_owners(&self) -> VecMapper<Self::Storage, Address>;

    #[storage_mapper("unclaimedTicketsForAddress")]
    fn uncalimed_tickets_for_address(
        &self,
        address: &Address,
    ) -> SafeSetMapper<Self::Storage, usize>;

    #[storage_mapper("shuffledTickets")]
    fn shuffled_tickets(&self) -> VecMapper<Self::Storage, usize>;
}
