#![no_std]

elrond_wasm::imports!();

mod setup;

mod ongoing_operation;
use ongoing_operation::*;

mod random;
use random::*;

#[elrond_wasm::derive::contract]
pub trait Launchpad: setup::SetupModule + ongoing_operation::OngoingOperationModule {
    // endpoints - owner-only

    #[only_owner]
    #[endpoint]
    fn add_tickets(
        &self,
        #[var_args] address_number_pairs: VarArgs<MultiArg2<Address, usize>>,
    ) -> SCResult<()> {
        let ongoing_operation = self.current_ongoing_operation().get();
        let mut index = match ongoing_operation {
            OngoingOperationType::None => 0,
            OngoingOperationType::AddTickets { index } => index,
            _ => return sc_error!("Another ongoing operation is in progress"),
        };

        let address_number_pairs_vec = address_number_pairs.into_vec();
        let nr_pairs = address_number_pairs_vec.len();

        let (first_buyer, first_nr_tickets) = address_number_pairs_vec[index].clone().into_tuple();
        index += 1;

        let gas_before = self.blockchain().get_gas_left();

        self.create_tickets(&first_buyer, first_nr_tickets);

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

                break;
            }
        }

        Ok(())
    }

    // endpoints

    // private

    fn create_tickets(&self, buyer: &Address, nr_tickets: usize) {
        for _ in 0..nr_tickets {
            self.ticket_owners().push(buyer);
        }
    }

    // storage

    // ticket ID -> address mapping
    #[storage_mapper("ticketOwners")]
    fn ticket_owners(&self) -> VecMapper<Self::Storage, Address>;
}
