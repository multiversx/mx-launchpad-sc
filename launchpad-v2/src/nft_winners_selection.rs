elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::elrond_codec::TopEncode;
use launchpad_common::{
    launch_stage::Flags,
    ongoing_operation::{OngoingOperationType, CONTINUE_OP, STOP_OP},
    random::Random,
};

const VEC_MAPPER_START_INDEX: usize = 1;

#[elrond_wasm::module]
pub trait NftWinnersSelectionModule:
    launchpad_common::launch_stage::LaunchStageModule
    + launchpad_common::config::ConfigModule
    + launchpad_common::ongoing_operation::OngoingOperationModule
    + launchpad_common::tickets::TicketsModule
    + crate::confirm_nft::ConfirmNftModule
{
    #[endpoint(selectNftWinners)]
    fn select_nft_winners(&self) -> OperationCompletionStatus {
        self.require_winner_selection_period();

        let flags_mapper = self.flags();
        let mut flags: Flags = flags_mapper.get();
        require!(
            flags.were_winners_selected,
            "Must select winners for base launchpad first"
        );
        require!(
            !flags.was_additional_step_completed,
            "Already selected NFT winners"
        );

        let mut rng: Random<Self::Api> = self.load_additional_selection_operation();

        let mut all_users_mapper = self.confirmed_nft_user_list();
        let users_mapper_vec = self.confirmed_list_to_vec_mapper();
        let mut nft_winners_mapper = self.nft_selection_winners();

        let mut users_left = users_mapper_vec.len();
        let mut winners_selected = nft_winners_mapper.len();
        let total_available_nfts = self.total_available_nfts().get();

        let run_result = self.run_while_it_has_gas(|| {
            let rand_index = rng.next_usize_in_range(VEC_MAPPER_START_INDEX, users_left + 1);
            let winner_addr = users_mapper_vec.get(rand_index);

            all_users_mapper.swap_remove(&winner_addr);
            let _ = nft_winners_mapper.insert(winner_addr);

            users_left -= 1;
            winners_selected += 1;

            if users_left == 0 || winners_selected == total_available_nfts {
                STOP_OP
            } else {
                CONTINUE_OP
            }
        });

        match run_result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                let mut encoded_rng = ManagedBuffer::new();
                let _ = rng.top_encode(&mut encoded_rng);

                self.save_progress(&OngoingOperationType::AdditionalSelection {
                    encoded_data: encoded_rng,
                });
            }
            OperationCompletionStatus::Completed => {
                flags.was_additional_step_completed = true;
                flags_mapper.set(&flags);

                let nft_cost = self.nft_cost().get();
                let claimable_nft_payment = nft_cost.amount * winners_selected as u32;
                self.claimable_nft_payment().set(&claimable_nft_payment);
            }
        }

        run_result
    }

    #[storage_mapper("nftSelectionWinners")]
    fn nft_selection_winners(&self) -> UnorderedSetMapper<ManagedAddress>;
}
