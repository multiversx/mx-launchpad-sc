#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod launch_stage;
mod setup;

mod ongoing_operation;
use ongoing_operation::*;

mod random;
use random::Random;

const FIRST_TICKET_ID: usize = 1;

type TicketStatus = bool;
const WINNING_TICKET: TicketStatus = true;

#[derive(TopEncode, TopDecode)]
pub struct TicketRange {
    pub first_id: usize,
    pub last_id: usize,
}

#[derive(TopEncode, TopDecode)]
pub struct TicketBatch {
    pub address: Address,
    pub nr_tickets: usize,
}

#[elrond_wasm::derive::contract]
pub trait Launchpad:
    launch_stage::LaunchStageModule + setup::SetupModule + ongoing_operation::OngoingOperationModule
{
    #[only_owner]
    #[endpoint(claimTicketPayment)]
    fn claim_ticket_payment(&self) -> SCResult<()> {
        self.require_claim_period()?;

        let owner = self.blockchain().get_caller();

        let ticket_payment_claimable_amount = self.claimable_ticket_payment().get();
        if ticket_payment_claimable_amount > 0 {
            let ticket_payment_token = self.ticket_payment_token().get();

            self.claimable_ticket_payment().clear();
            self.send().direct(
                &owner,
                &ticket_payment_token,
                0,
                &ticket_payment_claimable_amount,
                &[],
            );
        }

        Ok(())
    }

    #[only_owner]
    #[endpoint(addAddressToBlacklist)]
    fn add_address_to_blacklist(&self, address: Address) -> SCResult<()> {
        self.require_before_winner_selection()?;

        let nr_confirmed_tickets = self.nr_confirmed_tickets(&address).get();
        if nr_confirmed_tickets > 0 {
            self.refund_ticket_payment(&address, nr_confirmed_tickets);
            self.nr_confirmed_tickets(&address).clear();
        }

        self.blacklisted(&address).set(&true);

        Ok(())
    }

    #[only_owner]
    #[endpoint(removeAddressFromBlacklist)]
    fn remove_address_from_blacklist(&self, address: Address) -> SCResult<()> {
        self.require_before_winner_selection()?;

        self.blacklisted(&address).clear();

        Ok(())
    }

    #[only_owner]
    #[endpoint(addTickets)]
    fn add_tickets(
        &self,
        #[var_args] address_number_pairs: VarArgs<MultiArg2<Address, usize>>,
    ) -> SCResult<()> {
        self.require_add_tickets_period()?;

        for multi_arg in address_number_pairs.into_vec() {
            let (buyer, nr_tickets) = multi_arg.into_tuple();

            self.try_create_tickets(buyer, nr_tickets)?;
        }

        Ok(())
    }

    #[payable("*")]
    #[endpoint(confirmTickets)]
    fn confirm_tickets(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: Self::BigUint,
        nr_tickets_to_confirm: usize,
    ) -> SCResult<()> {
        self.require_confirmation_period()?;
        self.require_launchpad_tokens_deposited()?;

        let caller = self.blockchain().get_caller();
        require!(
            !self.is_user_blacklisted(&caller),
            "You have been put into the blacklist and may not confirm tickets"
        );

        let total_tickets = self.get_total_number_of_tickets_for_address(&caller);
        let nr_confirmed = self.nr_confirmed_tickets(&caller).get();
        let total_confirmed = nr_confirmed + nr_tickets_to_confirm;
        require!(
            total_confirmed <= total_tickets,
            "Trying to confirm too many tickets"
        );

        let ticket_payment_token = self.ticket_payment_token().get();
        let ticket_price = self.ticket_price().get();
        let total_ticket_price = Self::BigUint::from(nr_tickets_to_confirm) * ticket_price;
        require!(
            payment_token == ticket_payment_token,
            "Wrong payment token used"
        );
        require!(payment_amount == total_ticket_price, "Wrong amount sent");

        self.nr_confirmed_tickets(&caller).set(&total_confirmed);

        Ok(())
    }

    #[only_owner]
    #[endpoint(filterTickets)]
    fn filter_tickets(&self) -> SCResult<BoxedBytes> {
        self.require_winner_selection_period()?;
        require!(!self.were_tickets_filtered(), "Tickets already filtered");

        let last_ticket_id = self.last_ticket_id().get();
        let (mut first_ticket_id_in_batch, mut nr_removed) =
            self.load_filter_tickets_operation()?;

        let run_result = self.run_while_it_has_gas(|| {
            let ticket_batch = self.ticket_batch(first_ticket_id_in_batch).get();
            let address = &ticket_batch.address;
            let nr_tickets_in_batch = ticket_batch.nr_tickets;

            let nr_confirmed_tickets = self.nr_confirmed_tickets(address).get();
            if self.is_user_blacklisted(address) || nr_confirmed_tickets == 0 {
                self.ticket_range_for_address(address).clear();
                self.ticket_batch(first_ticket_id_in_batch).clear();
            } else if nr_removed > 0 || nr_confirmed_tickets < nr_tickets_in_batch {
                let new_first_id = first_ticket_id_in_batch - nr_removed;
                let new_last_id = new_first_id + nr_confirmed_tickets - 1;

                self.ticket_batch(first_ticket_id_in_batch).clear();

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
                self.tickets_filtered().set(&true);
            }
        };

        Ok(run_result.output_bytes().into())
    }

    #[only_owner]
    #[endpoint(selectWinners)]
    fn select_winners(&self) -> SCResult<BoxedBytes> {
        self.require_winner_selection_period()?;
        require!(self.were_tickets_filtered(), "Must filter tickets first");
        require!(!self.were_winners_selected(), "Winners already selected");

        let nr_winning_tickets = self.nr_winning_tickets().get();
        let last_ticket_position = self.get_total_tickets();

        let (mut rng, mut ticket_position) = self.load_select_winners_operation()?;
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
                    seed: rng.seed,
                    seed_index: rng.index,
                    ticket_position,
                });
            }
            OperationCompletionStatus::Completed => {
                self.winners_selected().set(&true);
            }
        };

        Ok(run_result.output_bytes().into())
    }

    #[endpoint(claimLaunchpadTokens)]
    fn claim_launchpad_tokens(&self) -> SCResult<()> {
        require!(self.winners_selected().get(), "Winners not selected yet");
        self.require_claim_period()?;

        let caller = self.blockchain().get_caller();
        require!(!self.has_user_claimed(&caller), "Already claimed");

        let ticket_range = self.try_get_ticket_range(&caller)?;
        let total_tickets = ticket_range.last_id - ticket_range.first_id + 1;
        let mut nr_redeemable_tickets = 0;

        for ticket_id in ticket_range.first_id..=ticket_range.last_id {
            let ticket_status = self.ticket_status(ticket_id).get();
            if ticket_status == WINNING_TICKET {
                self.ticket_status(ticket_id).clear();

                nr_redeemable_tickets += 1;
            }

            self.ticket_pos_to_id(ticket_id).clear();
        }

        self.nr_confirmed_tickets(&caller).clear();
        self.ticket_range_for_address(&caller).clear();
        self.ticket_batch(ticket_range.first_id).clear();

        if nr_redeemable_tickets > 0 {
            let ticket_price = self.ticket_price().get();
            let redeemed_ticket_cost = &Self::BigUint::from(nr_redeemable_tickets) * &ticket_price;
            self.claimable_ticket_payment()
                .update(|claimable_ticket_payment| {
                    *claimable_ticket_payment += redeemed_ticket_cost
                });

            self.total_claimed_tickets()
                .update(|total_claimed| *total_claimed += nr_redeemable_tickets);
        }

        self.claimed_tokens(&caller).set(&true);

        let nr_tickets_to_refund = total_tickets - nr_redeemable_tickets;
        self.refund_ticket_payment(&caller, nr_tickets_to_refund);
        self.send_launchpad_tokens(&caller, nr_redeemable_tickets);

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

        let ticket_range = self.ticket_range_for_address(&address).get();
        OptionalArg::Some((ticket_range.first_id, ticket_range.last_id).into())
    }

    #[view(getTotalNumberOfTicketsForAddress)]
    fn get_total_number_of_tickets_for_address(&self, address: &Address) -> usize {
        if self.ticket_range_for_address(address).is_empty() {
            return 0;
        }

        let ticket_range = self.ticket_range_for_address(address).get();
        ticket_range.last_id - ticket_range.first_id + 1
    }

    #[view(getWinningTicketIdsForAddress)]
    fn get_winning_ticket_ids_for_address(&self, address: Address) -> MultiResultVec<usize> {
        if self.ticket_range_for_address(&address).is_empty() {
            return MultiResultVec::new();
        }

        let mut ticket_ids = Vec::new();
        let ticket_range = self.ticket_range_for_address(&address).get();
        for ticket_id in ticket_range.first_id..=ticket_range.last_id {
            let actual_ticket_status = self.ticket_status(ticket_id).get();
            if actual_ticket_status == WINNING_TICKET {
                ticket_ids.push(ticket_id);
            }
        }

        ticket_ids.into()
    }

    #[view(getNumberOfWinningTicketsForAddress)]
    fn get_number_of_winning_tickets_for_address(&self, address: Address) -> usize {
        self.get_winning_ticket_ids_for_address(address).len()
    }

    // private

    fn try_create_tickets(&self, buyer: Address, nr_tickets: usize) -> SCResult<()> {
        require!(
            self.ticket_range_for_address(&buyer).is_empty(),
            "Duplicate entry for user"
        );

        let first_ticket_id = self.last_ticket_id().get() + 1;
        let last_ticket_id = first_ticket_id + nr_tickets - 1;

        self.ticket_range_for_address(&buyer).set(&TicketRange {
            first_id: first_ticket_id,
            last_id: last_ticket_id,
        });
        self.ticket_batch(first_ticket_id).set(&TicketBatch {
            address: buyer,
            nr_tickets,
        });
        self.last_ticket_id().set(&last_ticket_id);

        Ok(())
    }

    /// Fisher-Yates algorithm,
    /// each position i is swapped with a random one in range [i, n]
    fn shuffle_single_ticket(
        &self,
        rng: &mut Random<Self::CryptoApi>,
        current_ticket_position: usize,
        last_ticket_position: usize,
    ) {
        let rand_pos = rng.next_usize_in_range(current_ticket_position, last_ticket_position + 1);

        let winning_ticket_id = self.get_ticket_id_from_pos(rand_pos);
        self.ticket_status(winning_ticket_id).set(&WINNING_TICKET);

        let current_ticket_id = self.get_ticket_id_from_pos(current_ticket_position);
        self.ticket_pos_to_id(rand_pos).set(&current_ticket_id);
    }

    fn try_get_ticket_range(&self, address: &Address) -> SCResult<TicketRange> {
        require!(
            !self.ticket_range_for_address(address).is_empty(),
            "You have no tickets"
        );

        Ok(self.ticket_range_for_address(address).get())
    }

    fn get_ticket_id_from_pos(&self, ticket_pos: usize) -> usize {
        let mut ticket_id = self.ticket_pos_to_id(ticket_pos).get();
        if ticket_id == 0 {
            ticket_id = ticket_pos;
        }

        ticket_id
    }

    #[inline(always)]
    fn get_total_tickets(&self) -> usize {
        self.last_ticket_id().get()
    }

    #[inline(always)]
    fn has_user_claimed(&self, address: &Address) -> bool {
        self.claimed_tokens(address).get()
    }

    #[inline(always)]
    fn is_user_blacklisted(&self, address: &Address) -> bool {
        self.blacklisted(address).get()
    }

    #[inline(always)]
    fn were_tickets_filtered(&self) -> bool {
        self.tickets_filtered().get()
    }

    #[inline(always)]
    fn were_winners_selected(&self) -> bool {
        self.winners_selected().get()
    }

    fn refund_ticket_payment(&self, address: &Address, nr_tickets_to_refund: usize) {
        if nr_tickets_to_refund == 0 {
            return;
        }

        let ticket_price = self.ticket_price().get();
        let ticket_payment_token = self.ticket_payment_token().get();
        let ticket_payment_refund_amount = Self::BigUint::from(nr_tickets_to_refund) * ticket_price;

        self.send().direct(
            address,
            &ticket_payment_token,
            0,
            &ticket_payment_refund_amount,
            &[],
        );
    }

    fn send_launchpad_tokens(&self, address: &Address, nr_claimed_tickets: usize) {
        if nr_claimed_tickets == 0 {
            return;
        }

        let launchpad_token_id = self.launchpad_token_id().get();
        let tokens_per_winning_ticket = self.launchpad_tokens_per_winning_ticket().get();
        let launchpad_tokens_amount_to_send =
            Self::BigUint::from(nr_claimed_tickets) * tokens_per_winning_ticket;

        self.send().direct(
            address,
            &launchpad_token_id,
            0,
            &launchpad_tokens_amount_to_send,
            &[],
        );
    }

    // storage

    #[storage_mapper("ticketStatus")]
    fn ticket_status(&self, ticket_id: usize) -> SingleValueMapper<Self::Storage, TicketStatus>;

    #[storage_mapper("lastTicketId")]
    fn last_ticket_id(&self) -> SingleValueMapper<Self::Storage, usize>;

    #[storage_mapper("ticketBatch")]
    fn ticket_batch(&self, start_index: usize) -> SingleValueMapper<Self::Storage, TicketBatch>;

    #[storage_mapper("ticketRangeForAddress")]
    fn ticket_range_for_address(
        &self,
        address: &Address,
    ) -> SingleValueMapper<Self::Storage, TicketRange>;

    #[view(getNumberOfConfirmedTicketsForAddress)]
    #[storage_mapper("nrConfirmedTickets")]
    fn nr_confirmed_tickets(&self, address: &Address) -> SingleValueMapper<Self::Storage, usize>;

    // only used during shuffling. Default (0) means ticket pos = ticket ID.
    #[storage_mapper("ticketPosToId")]
    fn ticket_pos_to_id(&self, ticket_pos: usize) -> SingleValueMapper<Self::Storage, usize>;

    #[storage_mapper("claimableTicketPayment")]
    fn claimable_ticket_payment(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    // flags

    #[storage_mapper("ticketsFiltered")]
    fn tickets_filtered(&self) -> SingleValueMapper<Self::Storage, bool>;

    #[view(wereWinnersSelected)]
    #[storage_mapper("winnersSelected")]
    fn winners_selected(&self) -> SingleValueMapper<Self::Storage, bool>;

    #[view(hasUserClaimedTokens)]
    #[storage_mapper("claimedTokens")]
    fn claimed_tokens(&self, address: &Address) -> SingleValueMapper<Self::Storage, bool>;

    #[view(isUserBlacklisted)]
    #[storage_mapper("blacklisted")]
    fn blacklisted(&self, address: &Address) -> SingleValueMapper<Self::Storage, bool>;
}
