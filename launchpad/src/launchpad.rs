#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::elrond_codec::TopEncode;

mod setup;

mod launch_stage;
use launch_stage::*;

mod ongoing_operation;
use ongoing_operation::*;

mod random;
use random::Random;

mod ticket_status;
use ticket_status::TicketStatus;

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
    #[endpoint(forceClaimPeriodStart)]
    fn force_claim_period_start(&self) -> SCResult<()> {
        let total_tickets = self.get_total_tickets();
        self.total_confirmed_tickets().set(&total_tickets);

        Ok(())
    }

    #[only_owner]
    #[endpoint(addTickets)]
    fn add_tickets(
        &self,
        #[var_args] address_number_pairs: VarArgs<MultiArg2<Address, usize>>,
    ) -> SCResult<()> {
        self.require_no_ongoing_operation()?;
        self.require_stage(LaunchStage::AddTickets)?;

        let current_epoch = self.blockchain().get_block_epoch();
        let winner_selection_start_epoch = self.winner_selection_start_epoch().get();
        require!(
            current_epoch < winner_selection_start_epoch,
            "Cannot add more tickets, winner selection has started"
        );

        for multi_arg in address_number_pairs.into_vec() {
            let (buyer, nr_tickets) = multi_arg.into_tuple();

            self.try_create_tickets(&buyer, nr_tickets)?;
        }

        Ok(())
    }

    // endpoints

    #[endpoint(selectWinners)]
    fn select_winners(&self) -> SCResult<OperationCompletionStatus> {
        self.require_stage(LaunchStage::SelectWinners)?;

        let last_ticket_position = self.shuffled_tickets().len();
        let nr_winning_tickets = self.nr_winning_tickets().get();

        // dummy, will be overwritten in the Load part,
        // but the Rust compiler complains of possibly uninitialized variables otherwise
        let mut rng = Random::from_hash(self.crypto(), H256::zero(), 0);
        let mut ticket_position = 0;

        let run_result = self.run_while_it_has_gas(|gas_op| match gas_op {
            GasOp::Load(op) => match op {
                OngoingOperationType::None => {
                    rng = Random::from_seeds(
                        self.crypto(),
                        self.blockchain().get_prev_block_random_seed(),
                        self.blockchain().get_block_random_seed(),
                    );
                    ticket_position = VEC_MAPPER_START_INDEX;

                    LoopOp::Continue
                }
                OngoingOperationType::SelectWinners {
                    seed,
                    seed_index,
                    ticket_position: ticket_pos,
                } => {
                    rng = Random::from_hash(self.crypto(), seed, seed_index);
                    ticket_position = ticket_pos;

                    LoopOp::Continue
                }
                _ => LoopOp::Break,
            },
            GasOp::Continue => {
                let is_winning_ticket = ticket_position <= nr_winning_tickets;
                self.shuffle_single_ticket(
                    &mut rng,
                    ticket_position,
                    last_ticket_position,
                    is_winning_ticket,
                );
                ticket_position += 1;

                if ticket_position == last_ticket_position - 1 {
                    LoopOp::Break
                } else {
                    LoopOp::Continue
                }
            }
            GasOp::Save => LoopOp::Save(OngoingOperationType::SelectWinners {
                seed: rng.seed.clone(),
                seed_index: rng.index,
                ticket_position,
            }),
            GasOp::Completed => {
                self.start_confirmation_period(VEC_MAPPER_START_INDEX, nr_winning_tickets);

                LoopOp::Break
            }
        });

        run_result
    }

    #[payable("*")]
    #[endpoint(confirmTickets)]
    fn confirm_tickets(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: Self::BigUint,
        nr_tickets_to_confirm: usize,
    ) -> SCResult<()> {
        self.require_stage(LaunchStage::ConfirmTickets)?;

        let caller = self.blockchain().get_caller();
        require!(
            !self.blacklist().contains(&caller),
            "You have been put into the blacklist and may not confirm tickets"
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

        self.total_confirmed_tickets()
            .update(|confirmed| *confirmed += actual_confirmed_tickets);

        Ok(())
    }

    #[endpoint(selectNewWinners)]
    fn select_new_winners(&self) -> SCResult<OperationCompletionStatus> {
        self.require_stage(LaunchStage::SelectNewWinners)?;

        let (prev_first_winning_ticket_position, prev_last_winning_ticket_position) =
            self.winning_tickets_range().get();
        let winning_tickets =
            prev_last_winning_ticket_position - prev_first_winning_ticket_position + 1;
        let confirmed_tickets = self.total_confirmed_tickets().get();
        let remaining_tickets = winning_tickets - confirmed_tickets;

        let new_first_winning_ticket_position = prev_first_winning_ticket_position + 1;
        let new_last_winning_ticket_position =
            new_first_winning_ticket_position + remaining_tickets - 1;
        let last_valid_ticket_id = self.get_total_tickets();

        require!(
            new_last_winning_ticket_position <= last_valid_ticket_id,
            "Cannot select new winners, reached end of range"
        );

        let next_generation = self.current_generation().get() + 1;
        let mut current_ticket_position = 0;

        let run_result = self.run_while_it_has_gas(|gas_op| match gas_op {
            GasOp::Load(op) => match op {
                OngoingOperationType::None => {
                    current_ticket_position = new_first_winning_ticket_position;

                    LoopOp::Continue
                }
                OngoingOperationType::SelectNewWinners { ticket_position } => {
                    current_ticket_position = ticket_position;

                    LoopOp::Continue
                }
                _ => LoopOp::Break,
            },
            GasOp::Continue => {
                let winning_ticket_id = self.shuffled_tickets().get(current_ticket_position);
                self.set_ticket_status(
                    winning_ticket_id,
                    TicketStatus::Winning {
                        generation: next_generation,
                    },
                );
                current_ticket_position += 1;

                if current_ticket_position == new_last_winning_ticket_position {
                    LoopOp::Break
                } else {
                    LoopOp::Continue
                }
            }
            GasOp::Save => LoopOp::Save(OngoingOperationType::SelectNewWinners {
                ticket_position: current_ticket_position,
            }),
            GasOp::Completed => {
                self.winning_tickets_range().set(&(
                    new_first_winning_ticket_position,
                    new_last_winning_ticket_position,
                ));
                self.nr_winning_tickets().set(&remaining_tickets);
                self.total_confirmed_tickets().clear();

                self.start_confirmation_period(
                    new_first_winning_ticket_position,
                    new_last_winning_ticket_position,
                );

                LoopOp::Break
            }
        });

        run_result
    }

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens(&self) -> SCResult<()> {
        self.require_stage(LaunchStage::Claim)?;

        let caller = self.blockchain().get_caller();
        require!(
            !self.blacklist().contains(&caller),
            "You have been put into the blacklist and may not claim tokens"
        );

        let (first_ticket_id, last_ticket_id) = self.try_get_ticket_range(&caller)?;
        let mut nr_redeemed_tickets = 0u32;

        for ticket_id in first_ticket_id..=last_ticket_id {
            let ticket_status = self.ticket_status().get(ticket_id);
            if !ticket_status.is_confirmed() {
                continue;
            }

            self.set_ticket_status(ticket_id, TicketStatus::Redeemed);
            nr_redeemed_tickets += 1;
        }

        require!(nr_redeemed_tickets > 0, "No tickets to redeem");

        let launchpad_token_id = self.launchpad_token_id().get();
        let tokens_per_winning_ticket = self.launchpad_tokens_per_winning_ticket().get();
        let amount_to_send = Self::BigUint::from(nr_redeemed_tickets) * tokens_per_winning_ticket;

        self.send()
            .direct(&caller, &launchpad_token_id, 0, &amount_to_send, &[]);

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
        let current_generation = self.current_generation().get();

        for ticket_id in first_ticket_id..=last_ticket_id {
            let ticket_status = self.ticket_status().get(ticket_id);
            if ticket_status.is_winning(current_generation) {
                nr_winning_tickets += 1;
            }
        }

        nr_winning_tickets
    }

    #[view(getLaunchStage)]
    fn get_launch_stage(&self) -> LaunchStage {
        let current_epoch = self.blockchain().get_block_epoch();

        let total_tickets = self.get_total_tickets();
        let total_confirmed_tickets = self.get_total_confirmed_tickets();
        if total_confirmed_tickets == total_tickets {
            let claim_start_epoch = self.claim_start_epoch().get();
            if current_epoch >= claim_start_epoch {
                return LaunchStage::Claim;
            } else {
                return LaunchStage::WaitBeforeClaim;
            }
        }

        let winner_selection_start_epoch = self.winner_selection_start_epoch().get();
        if current_epoch < winner_selection_start_epoch {
            return LaunchStage::AddTickets;
        }

        // confirmation period start is always set after the first SelectWinners
        if self.confirmation_period_start_epoch().is_empty() {
            return LaunchStage::SelectWinners;
        }

        let confirmation_period_start_epoch = self.confirmation_period_start_epoch().get();
        let confirmation_period_in_epochs = self.confirmation_period_in_epochs().get();
        let confiration_period_end_epoch =
            confirmation_period_start_epoch + confirmation_period_in_epochs;
        if current_epoch < confiration_period_end_epoch {
            return LaunchStage::ConfirmTickets;
        }

        LaunchStage::SelectNewWinners
    }

    // private

    fn try_create_tickets(&self, buyer: &Address, nr_tickets: usize) -> SCResult<()> {
        require!(
            self.ticket_range_for_address(buyer).is_empty(),
            "Duplicate entry for user"
        );

        let first_ticket_id = self.shuffled_tickets().len() + 1;
        let last_ticket_id = first_ticket_id + nr_tickets - 1;
        self.ticket_range_for_address(buyer)
            .set(&(first_ticket_id, last_ticket_id));

        for ticket_id in first_ticket_id..=last_ticket_id {
            self.shuffled_tickets().push(&ticket_id);
        }

        Ok(())
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

        self.winning_tickets_range()
            .set(&(first_winning_ticket_position, last_winning_ticket_position));
        self.current_generation()
            .update(|current_generation| *current_generation += 1);
    }

    fn try_get_ticket_range(&self, address: &Address) -> SCResult<(usize, usize)> {
        require!(
            !self.ticket_range_for_address(address).is_empty(),
            "You have no tickets"
        );

        Ok(self.ticket_range_for_address(address).get())
    }

    fn get_total_tickets(&self) -> usize {
        self.shuffled_tickets().len()
    }

    fn get_total_confirmed_tickets(&self) -> usize {
        self.total_confirmed_tickets().get()
    }

    fn require_stage(&self, expected_stage: LaunchStage) -> SCResult<()> {
        let actual_stage = self.get_launch_stage();

        require!(
            actual_stage == expected_stage,
            "Cannot call this endpoint, SC is in a different stage"
        );

        Ok(())
    }

    // storage

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

    #[storage_mapper("currentGeneration")]
    fn current_generation(&self) -> SingleValueMapper<Self::Storage, u8>;

    #[storage_mapper("totalConfirmedTickets")]
    fn total_confirmed_tickets(&self) -> SingleValueMapper<Self::Storage, usize>;
}
