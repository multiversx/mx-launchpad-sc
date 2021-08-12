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

use crate::setup::SetupModule;

const VEC_MAPPER_START_INDEX: usize = 1;
const FIRST_GENERATION: u8 = 1;

#[elrond_wasm::derive::contract]
pub trait Launchpad: setup::SetupModule + ongoing_operation::OngoingOperationModule {
    // endpoints - owner-only

    #[only_owner]
    #[endpoint(claimTicketPayment)]
    fn claim_ticket_payment(&self) -> SCResult<()> {
        let ticket_payment_token = self.ticket_payment_token().get();
        let sc_balance = self.blockchain().get_sc_balance(&ticket_payment_token, 0);
        let owner = self.blockchain().get_caller();

        if sc_balance > 0 {
            self.send()
                .direct(&owner, &ticket_payment_token, 0, &sc_balance, &[]);
        }

        Ok(())
    }

    #[only_owner]
    #[endpoint(addTickets)]
    fn add_tickets(
        &self,
        #[var_args] address_number_pairs: VarArgs<MultiArg2<Address, usize>>,
    ) -> SCResult<OperationCompletionStatus> {
        let ongoing_operation = self.current_ongoing_operation().get();
        let mut index = match ongoing_operation {
            OngoingOperationType::None => {
                require!(self.ticket_owners().is_empty(), "Cannot add more tickets");
                0
            }
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
        let nr_winning_tickets = self.nr_winning_tickets().get();

        let gas_before = self.blockchain().get_gas_left();

        let is_winning_ticket = ticket_position <= nr_winning_tickets;
        self.shuffle_single_ticket(
            &mut rng,
            ticket_position,
            last_ticket_position,
            is_winning_ticket,
        );
        ticket_position += 1;

        let gas_after = self.blockchain().get_gas_left();
        let gas_per_iteration = gas_before - gas_after;

        while ticket_position < last_ticket_position - 1 {
            if self.can_continue_operation(gas_per_iteration) {
                let is_winning_ticket = ticket_position <= nr_winning_tickets;
                self.shuffle_single_ticket(
                    &mut rng,
                    ticket_position,
                    last_ticket_position,
                    is_winning_ticket,
                );
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

        self.start_confirmation_period(VEC_MAPPER_START_INDEX, nr_winning_tickets);

        Ok(OperationCompletionStatus::Completed)
    }

    #[payable("*")]
    #[endpoint(confirmTickets)]
    fn confirm_tickets(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: Self::BigUint,
        nr_tickets_to_confirm: usize,
    ) -> SCResult<OperationCompletionStatus> {
        let current_epoch = self.blockchain().get_block_epoch();
        let claim_start_epoch = self.claim_period_start_epoch().get();
        let claim_period_in_epochs = self.claim_period_in_epochs().get();
        let claim_end_epoch = claim_start_epoch + claim_period_in_epochs;

        if current_epoch > claim_end_epoch {
            self.select_new_winners()?;
            return Ok(OperationCompletionStatus::InterruptedBeforeOutOfGas);
        }

        let caller = self.blockchain().get_caller();
        require!(
            !self.blacklist().contains(&caller),
            "You have been put into the blacklist and may not confirm tickets"
        );

        let confirmation_start_epoch = self.confirmation_period_start_epoch().get();
        let confirmation_period_in_epochs = self.confirmation_period_in_epochs().get();
        let confirmation_end_epoch = confirmation_start_epoch + confirmation_period_in_epochs;

        require!(
            current_epoch > confirmation_start_epoch,
            "Confirmation period has not started yet"
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

        let (first_ticket_id, last_ticket_id) = self.try_get_ticket_range(&caller)?;
        let nr_tickets = last_ticket_id - first_ticket_id + 1;

        require!(
            nr_tickets >= nr_tickets_to_confirm,
            "Trying to confirm too many tickets"
        );

        let current_generation = self.current_generation().get();
        let mut actual_confirmed_tickets = 0;

        for ticket_id in first_ticket_id..=last_ticket_id {
            let ticket_status = self.ticket_status().get(ticket_id);
            if !ticket_status.is_winning(current_generation) {
                continue;
            }

            self.set_ticket_status(
                ticket_id,
                TicketStatus::Confirmed {
                    generation: current_generation,
                },
            );
            actual_confirmed_tickets += 1;

            if actual_confirmed_tickets == nr_tickets_to_confirm {
                break;
            }
        }

        require!(
            actual_confirmed_tickets == nr_tickets_to_confirm,
            "Couldn't confirm all tickets"
        );

        Ok(OperationCompletionStatus::Completed)
    }

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens(&self) -> SCResult<OperationCompletionStatus> {
        let caller = self.blockchain().get_caller();
        require!(
            !self.blacklist().contains(&caller),
            "You have been put into the blacklist and may not claim tokens"
        );

        let current_epoch = self.blockchain().get_block_epoch();
        let claim_start_epoch = self.claim_period_start_epoch().get();
        let claim_period_in_epochs = self.claim_period_in_epochs().get();
        let claim_end_epoch = claim_start_epoch + claim_period_in_epochs;

        require!(
            current_epoch > claim_start_epoch,
            "Claim period has not started yet"
        );

        if current_epoch > claim_end_epoch {
            self.select_new_winners()?;
            return Ok(OperationCompletionStatus::InterruptedBeforeOutOfGas);
        }

        let (first_ticket_id, last_ticket_id) = self.try_get_ticket_range(&caller)?;
        let current_generation = self.current_generation().get();
        let mut nr_redeemed_tickets = 0;

        for ticket_id in first_ticket_id..=last_ticket_id {
            let ticket_status = self.ticket_status().get(ticket_id);
            if !ticket_status.is_confirmed(current_generation) {
                continue;
            }

            self.set_ticket_status(ticket_id, TicketStatus::Redeemed);
            nr_redeemed_tickets += 1;
        }

        require!(nr_redeemed_tickets > 0, "No tickets to redeem");

        self.nr_redeemed_tickets()
            .update(|redeemed| *redeemed += nr_redeemed_tickets);

        let launchpad_token_id = self.launchpad_token_id().get();
        let tokens_per_winning_ticket = self.launchpad_tokens_per_winning_ticket().get();
        let amount_to_send = Self::BigUint::from(nr_redeemed_tickets) * tokens_per_winning_ticket;

        self.send()
            .direct(&caller, &launchpad_token_id, 0, &amount_to_send, &[]);

        Ok(OperationCompletionStatus::Completed)
    }

    // views

    #[view(getNumberOfWinningTicketsForAddress)]
    fn get_number_of_winning_tickets_for_address(&self, address: Address) -> usize {
        if self.ticket_range_for_address(&address).is_empty() {
            return 0;
        }

        let mut nr_winning_tickets = 0;
        let (first_ticket_id, last_ticket_id) = self.ticket_range_for_address(&address).get();
        let current_generation = self.current_generation().get();

        for ticket_id in first_ticket_id..=last_ticket_id {
            let ticket_status = self.ticket_status().get(ticket_id);
            if ticket_status.is_winning(current_generation) {
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
            self.shuffled_tickets().push(&ticket_id);
        }
    }

    /// Fisher-Yates algorithm,
    /// each position is swapped with a random one that's after it.
    fn shuffle_single_ticket(
        &self,
        rng: &mut Random<Self::CryptoApi>,
        current_ticket_position: usize,
        last_ticket_position: usize,
        is_winning_ticket: bool,
    ) {
        let rand_index =
            rng.next_usize_in_range(current_ticket_position + 1, last_ticket_position + 1);
        self.swap(self.shuffled_tickets(), current_ticket_position, rand_index);

        if is_winning_ticket {
            let winning_ticket_id = self.shuffled_tickets().get(current_ticket_position);
            self.set_ticket_status(
                winning_ticket_id,
                TicketStatus::Winning {
                    generation: FIRST_GENERATION,
                },
            );
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

    fn set_ticket_status(&self, ticket_id: usize, status: TicketStatus) {
        self.ticket_status().set(ticket_id, &status);
    }

    fn start_confirmation_period(
        &self,
        first_winning_ticket_position: usize,
        last_winning_ticket_position: usize,
    ) {
        let current_epoch = self.blockchain().get_block_epoch();
        self.confirmation_period_start_epoch().set(&current_epoch);

        let confirmation_period_in_epochs = self.confirmation_period_in_epochs().get();
        let claim_period_start_epoch = current_epoch + confirmation_period_in_epochs + 1;
        self.claim_period_start_epoch()
            .set(&claim_period_start_epoch);

        self.winning_tickets_range()
            .set(&(first_winning_ticket_position, last_winning_ticket_position));
        self.current_generation()
            .update(|current_generation| *current_generation += 1);
    }

    fn select_new_winners(&self) -> SCResult<()> {
        let (prev_first_winning_ticket_position, prev_last_winning_ticket_position) =
            self.winning_tickets_range().get();
        let winning_tickets =
            prev_last_winning_ticket_position - prev_first_winning_ticket_position + 1;
        let redeemed_tickets = self.nr_redeemed_tickets().get();
        let remaining_tickets = winning_tickets - redeemed_tickets;

        let new_first_winning_ticket_position = prev_first_winning_ticket_position + 1;
        let new_last_winning_ticket_position =
            new_first_winning_ticket_position + remaining_tickets - 1;

        let next_generation = self.current_generation().get() + 1;

        let ongoing_operation = self.current_ongoing_operation().get();
        let mut current_ticket_position = match ongoing_operation {
            OngoingOperationType::None => new_first_winning_ticket_position,
            OngoingOperationType::RestartConfirmationPeriod { ticket_position } => ticket_position,
            _ => return sc_error!("Another ongoing operation is in progress"),
        };

        let gas_before = self.blockchain().get_gas_left();

        let mut winning_ticket_id = self.shuffled_tickets().get(current_ticket_position);
        self.set_ticket_status(
            winning_ticket_id,
            TicketStatus::Winning {
                generation: next_generation,
            },
        );
        current_ticket_position += 1;

        let gas_after = self.blockchain().get_gas_left();
        let gas_per_iteration = gas_before - gas_after;

        while current_ticket_position <= new_last_winning_ticket_position {
            if self.can_continue_operation(gas_per_iteration) {
                winning_ticket_id = self.shuffled_tickets().get(current_ticket_position);
                self.set_ticket_status(
                    winning_ticket_id,
                    TicketStatus::Winning {
                        generation: next_generation,
                    },
                );
                current_ticket_position += 1;
            } else {
                self.save_progress(&OngoingOperationType::RestartConfirmationPeriod {
                    ticket_position: current_ticket_position,
                })
            }
        }

        self.winning_tickets_range().set(&(
            new_first_winning_ticket_position,
            new_last_winning_ticket_position,
        ));
        self.nr_winning_tickets().set(&remaining_tickets);
        self.nr_redeemed_tickets().clear();

        self.start_confirmation_period(
            new_first_winning_ticket_position,
            new_last_winning_ticket_position,
        );

        Ok(())
    }

    fn try_get_ticket_range(&self, address: &Address) -> SCResult<(usize, usize)> {
        require!(
            !self.ticket_range_for_address(address).is_empty(),
            "You have no tickets"
        );

        Ok(self.ticket_range_for_address(address).get())
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

    #[storage_mapper("winningTicketsRange")]
    fn winning_tickets_range(&self) -> SingleValueMapper<Self::Storage, (usize, usize)>;

    #[storage_mapper("shuffledTickets")]
    fn shuffled_tickets(&self) -> VecMapper<Self::Storage, usize>;

    #[view(getConfirmationPeriodStartEpoch)]
    #[storage_mapper("confirmationPeriodStartEpoch")]
    fn confirmation_period_start_epoch(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[view(getClaimPeriodStartEpoch)]
    #[storage_mapper("claimPeriodStartEpoch")]
    fn claim_period_start_epoch(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[storage_mapper("currentGeneration")]
    fn current_generation(&self) -> SingleValueMapper<Self::Storage, u8>;

    #[storage_mapper("nrRedeemedTickets")]
    fn nr_redeemed_tickets(&self) -> SingleValueMapper<Self::Storage, usize>;
}
