elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::{elrond_codec::TopEncode, storage::StorageKey};
use launchpad_common::{
    ongoing_operation::{OngoingOperationType, CONTINUE_OP, STOP_OP},
    random::Random,
    tickets::{TicketRange, WINNING_TICKET},
};

const VEC_MAPPER_START_INDEX: usize = 1;

#[derive(TopEncode, TopDecode)]
pub struct GuaranteedTicketsSelectionOperation<M: ManagedTypeApi + CryptoApi> {
    pub rng: Random<M>,
    pub leftover_tickets: usize,
}

impl<M: ManagedTypeApi + CryptoApi> Default for GuaranteedTicketsSelectionOperation<M> {
    fn default() -> Self {
        Self {
            rng: Random::default(),
            leftover_tickets: 0,
        }
    }
}

#[elrond_wasm::module]
pub trait GuaranteedTicketWinnersModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::ongoing_operation::OngoingOperationModule
    + launchpad_common::tickets::TicketsModule
{
    #[endpoint(distributeGuaranteedTickets)]
    fn distribute_guaranteed_tickets(&self) -> OperationCompletionStatus {
        self.require_winner_selection_period();

        let flags_mapper = self.flags();
        let mut flags = flags_mapper.get();
        require!(
            flags.were_winners_selected,
            "Must select winners for base launchpad first"
        );
        require!(
            !flags.was_additional_step_completed,
            "Already distributed tickets"
        );

        let mut current_operation: GuaranteedTicketsSelectionOperation<Self::Api> =
            self.load_additional_selection_operation();
        let leftover_tickets = &mut current_operation.leftover_tickets;
        let max_tier_tickets = self.max_tier_tickets().get();

        let mut users_whitelist = self.max_tier_users();
        let users_list_vec = self.get_users_list_vec_mapper();
        let mut users_left = users_list_vec.len();

        let run_result = if users_left > 0 {
            self.run_while_it_has_gas(|| {
                let current_user = users_list_vec.get(VEC_MAPPER_START_INDEX);
                let _ = users_whitelist.swap_remove(&current_user);
                users_left -= 1;

                let user_confirmed_tickets = self.nr_confirmed_tickets(&current_user).get();
                if user_confirmed_tickets == max_tier_tickets {
                    let ticket_range = self.ticket_range_for_address(&current_user).get();
                    if !self.has_any_winning_tickets(&ticket_range) {
                        self.ticket_status(ticket_range.first_id)
                            .set(WINNING_TICKET);
                    } else {
                        *leftover_tickets += 1;
                    }
                } else {
                    *leftover_tickets += 1;
                }

                if users_left == 0 {
                    STOP_OP
                } else {
                    CONTINUE_OP
                }
            })
        } else {
            OperationCompletionStatus::Completed
        };

        if run_result == OperationCompletionStatus::InterruptedBeforeOutOfGas {
            let mut encoded_data = ManagedBuffer::new();
            let _ = current_operation.top_encode(&mut encoded_data);
            self.save_progress(&OngoingOperationType::AdditionalSelection { encoded_data });

            return run_result;
        }

        // let rng = &mut current_operation.rng;
        // let nr_original_winning_tickets = self.nr_winning_tickets().get();

        run_result
    }

    fn has_any_winning_tickets(&self, ticket_range: &TicketRange) -> bool {
        for ticket_id in ticket_range.first_id..=ticket_range.last_id {
            let ticket_status = self.ticket_status(ticket_id).get();
            if ticket_status == WINNING_TICKET {
                return true;
            }
        }

        false
    }

    #[inline]
    fn get_users_list_vec_mapper(&self) -> VecMapper<ManagedAddress> {
        VecMapper::new(StorageKey::new(b"maxTierUsers"))
    }

    #[storage_mapper("maxTierTickets")]
    fn max_tier_tickets(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("maxTierUsers")]
    fn max_tier_users(&self) -> UnorderedSetMapper<ManagedAddress>;
}
