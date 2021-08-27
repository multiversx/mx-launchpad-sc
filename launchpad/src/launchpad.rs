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
        let total_winning_tickets = self.nr_winning_tickets().get();
        self.total_confirmed_tickets().set(&total_winning_tickets);

        Ok(())
    }

    #[only_owner]
    #[endpoint(addAddressToBlacklist)]
    fn add_address_to_blacklist(&self, address: Address) -> SCResult<()> {
        self.blacklist().insert(address);

        Ok(())
    }

    #[only_owner]
    #[endpoint(removeAddressFromBlacklist)]
    fn remove_address_from_blacklist(&self, address: Address) -> SCResult<()> {
        self.blacklist().remove(&address);

        Ok(())
    }

    #[only_owner]
    #[endpoint(refundConfirmedTickets)]
    fn refund_confirmed_tickets(&self, address: Address) -> SCResult<()> {
        require!(
            self.blacklist().contains(&address),
            "Can only refund for users that have been put in blacklist"
        );

        let (first_ticket_id, last_ticket_id) = self.ticket_range_for_address(&address).get();
        let mut nr_refunded_tickets = 0;

        for ticket_id in first_ticket_id..=last_ticket_id {
            let ticket_status = self.ticket_status(ticket_id).get();
            if !ticket_status.is_confirmed(None) {
                continue;
            }

            self.ticket_status(ticket_id).set(&TicketStatus::None);

            nr_refunded_tickets += 1;
        }

        self.total_confirmed_tickets()
            .update(|confirmed| *confirmed -= nr_refunded_tickets);

        let ticket_paymemt_token = self.ticket_payment_token().get();
        let ticket_price = self.ticket_price().get();
        let amount_to_refund = ticket_price * nr_refunded_tickets.into();

        self.send()
            .direct(&address, &ticket_paymemt_token, 0, &amount_to_refund, &[]);

        Ok(())
    }

    #[only_owner]
    #[endpoint(addTickets)]
    fn add_tickets(
        &self,
        #[var_args] address_number_pairs: VarArgs<MultiArg2<Address, usize>>,
    ) -> SCResult<()> {
        self.require_stage(LaunchStage::AddTickets)?;

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
        let (mut rng, mut ticket_position, nr_winning_tickets) =
            self.load_select_winners_operation()?;

        require!(
            nr_winning_tickets <= last_ticket_position,
            "Cannot select winners, number of winning tickets is higher than total amount of tickets"
        );

        let run_result = self.run_while_it_has_gas(|| {
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
        })?;

        match run_result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                self.save_progress(&OngoingOperationType::SelectWinners {
                    seed: rng.seed,
                    seed_index: rng.index,
                    ticket_position,
                    nr_winning_tickets,
                });
            }
            OperationCompletionStatus::Completed => {
                self.start_confirmation_period(VEC_MAPPER_START_INDEX, nr_winning_tickets);
            }
        };

        Ok(run_result)
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
        let winning_tickets = self.get_tickets_with_status(
            &caller,
            TicketStatus::Winning {
                generation: current_generation,
            },
        );

        require!(
            nr_tickets_to_confirm <= winning_tickets.len(),
            "Trying to confirm too many tickets"
        );

        for winning_ticket_id in &winning_tickets[..nr_tickets_to_confirm] {
            self.ticket_status(*winning_ticket_id)
                .set(&TicketStatus::Confirmed {
                    generation: current_generation,
                });
        }

        self.total_confirmed_tickets()
            .update(|confirmed| *confirmed += nr_tickets_to_confirm);

        Ok(())
    }

    #[endpoint(selectNewWinners)]
    fn select_new_winners(&self) -> SCResult<OperationCompletionStatus> {
        self.require_stage(LaunchStage::SelectNewWinners)?;

        let (_, prev_last_winning_ticket_position) = self.winning_tickets_range().get();
        let new_first_winning_ticket_position = prev_last_winning_ticket_position + 1;

        let (mut current_ticket_position, winning_tickets) =
            self.load_select_new_winners_operation(new_first_winning_ticket_position)?;
        let confirmed_tickets = self.total_confirmed_tickets().get();
        let remaining_tickets = winning_tickets - confirmed_tickets;

        let new_last_winning_ticket_position =
            new_first_winning_ticket_position + remaining_tickets - 1;
        let last_valid_ticket_id = self.get_total_tickets();

        require!(
            new_last_winning_ticket_position <= last_valid_ticket_id,
            "Cannot select new winners, reached end of range"
        );

        let next_generation = self.current_generation().get() + 1;

        let run_result = self.run_while_it_has_gas(|| {
            let winning_ticket_id = self.shuffled_tickets().get(current_ticket_position);

            self.ticket_status(winning_ticket_id)
                .set(&TicketStatus::Winning {
                    generation: next_generation,
                });

            current_ticket_position += 1;
            if current_ticket_position <= new_last_winning_ticket_position {
                LoopOp::Continue
            } else {
                LoopOp::Break
            }
        })?;

        match run_result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                self.save_progress(&OngoingOperationType::SelectNewWinners {
                    ticket_position: current_ticket_position,
                    nr_winning_tickets: winning_tickets,
                });
            }
            OperationCompletionStatus::Completed => {
                self.winning_tickets_range().set(&(
                    new_first_winning_ticket_position,
                    new_last_winning_ticket_position,
                ));

                self.start_confirmation_period(
                    new_first_winning_ticket_position,
                    new_last_winning_ticket_position,
                );
            }
        };

        Ok(run_result)
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
            let ticket_status = self.ticket_status(ticket_id).get();
            if !ticket_status.is_confirmed(None) {
                continue;
            }

            self.ticket_status(ticket_id).set(&TicketStatus::Redeemed);

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

    // range is [min, max], both inclusive
    #[view(getTicketRangeForAddress)]
    fn get_ticket_range_for_address(
        &self,
        address: Address,
    ) -> OptionalResult<MultiResult2<usize, usize>> {
        if self.ticket_range_for_address(&address).is_empty() {
            return OptionalResult::None;
        }

        OptionalArg::Some(self.ticket_range_for_address(&address).get().into())
    }

    #[view(getWinningTicketIdsForAddress)]
    fn get_winning_ticket_ids_for_address(&self, address: Address) -> MultiResultVec<usize> {
        let current_generation = self.current_generation().get();
        let winning_ticket_ids = self.get_tickets_with_status(
            &address,
            TicketStatus::Winning {
                generation: current_generation,
            },
        );

        winning_ticket_ids.into()
    }

    #[view(getConfirmedTicketIdsForAddress)]
    fn get_confirmed_ticket_ids_for_address(&self, address: Address) -> MultiResultVec<usize> {
        let current_generation = self.current_generation().get();
        let confirmed_ticket_ids = self.get_tickets_with_status(
            &address,
            TicketStatus::Confirmed {
                generation: current_generation,
            },
        );

        confirmed_ticket_ids.into()
    }

    #[view(getNumberOfWinningTicketsForAddress)]
    fn get_number_of_winning_tickets_for_address(&self, address: Address) -> usize {
        self.get_winning_ticket_ids_for_address(address).len()
    }

    #[view(getNumberOfConfirmedTicketsForAddress)]
    fn get_number_of_confirmed_tickets_for_address(&self, address: Address) -> usize {
        self.get_confirmed_ticket_ids_for_address(address).len()
    }

    #[view(getLaunchStage)]
    fn get_launch_stage(&self) -> LaunchStage {
        let current_epoch = self.blockchain().get_block_epoch();

        let total_winning_tickets = self.nr_winning_tickets().get();
        let total_confirmed_tickets = self.get_total_confirmed_tickets();
        if total_confirmed_tickets >= total_winning_tickets {
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

        let current_generation = self.current_generation().get();
        if current_generation < FIRST_GENERATION {
            return LaunchStage::SelectWinners;
        }

        let confirmation_period_start_epoch = self.confirmation_period_start_epoch().get();
        let confirmation_period_in_epochs = self.confirmation_period_in_epochs().get();
        let confiration_period_end_epoch =
            confirmation_period_start_epoch + confirmation_period_in_epochs;
        if current_epoch < confirmation_period_start_epoch {
            return LaunchStage::WaitBeforeConfirmation;
        }
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

            self.ticket_status(winning_ticket_id)
                .set(&TicketStatus::Winning {
                    generation: FIRST_GENERATION,
                });
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

    fn start_confirmation_period(
        &self,
        first_winning_ticket_position: usize,
        last_winning_ticket_position: usize,
    ) {
        let current_epoch = self.blockchain().get_block_epoch();
        let confirmation_start = self.confirmation_period_start_epoch().get();

        // done for the cases where the owner intentionally delays confirmation period
        // in which case we don't overwrite
        if current_epoch > confirmation_start {
            self.confirmation_period_start_epoch().set(&current_epoch);
        }

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

    fn get_tickets_with_status(
        &self,
        address: &Address,
        expected_ticket_status: TicketStatus,
    ) -> Vec<usize> {
        if self.ticket_range_for_address(address).is_empty() {
            return Vec::new();
        }

        let mut ticket_ids = Vec::new();
        let (first_ticket_id, last_ticket_id) = self.ticket_range_for_address(address).get();

        for ticket_id in first_ticket_id..=last_ticket_id {
            let actual_ticket_status = self.ticket_status(ticket_id).get();
            if actual_ticket_status == expected_ticket_status {
                ticket_ids.push(ticket_id);
            }
        }

        ticket_ids
    }

    // storage

    #[storage_mapper("ticketStatus")]
    fn ticket_status(&self, ticket_id: usize) -> SingleValueMapper<Self::Storage, TicketStatus>;

    #[storage_mapper("ticketRangeForAddress")]
    fn ticket_range_for_address(
        &self,
        address: &Address,
    ) -> SingleValueMapper<Self::Storage, (usize, usize)>;

    #[storage_mapper("winningTicketsRange")]
    fn winning_tickets_range(&self) -> SingleValueMapper<Self::Storage, (usize, usize)>;

    #[storage_mapper("shuffledTickets")]
    fn shuffled_tickets(&self) -> VecMapper<Self::Storage, usize>;

    #[storage_mapper("currentGeneration")]
    fn current_generation(&self) -> SingleValueMapper<Self::Storage, u8>;

    #[storage_mapper("totalConfirmedTickets")]
    fn total_confirmed_tickets(&self) -> SingleValueMapper<Self::Storage, usize>;

    #[storage_mapper("blacklist")]
    fn blacklist(&self) -> SafeSetMapper<Self::Storage, Address>;
}
