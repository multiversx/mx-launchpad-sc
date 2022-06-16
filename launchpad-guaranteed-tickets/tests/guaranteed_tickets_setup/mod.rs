use elrond_wasm::types::{
    Address, EgldOrEsdtTokenIdentifier, MultiValueEncoded, OperationCompletionStatus,
};
use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    tx_mock::TxResult,
    DebugApi,
};
use launchpad_common::{
    config::ConfigModule,
    launch_stage::{Flags, LaunchStageModule},
    tickets::{TicketsModule, WINNING_TICKET},
    user_interactions::UserInteractionsModule,
    winner_selection::WinnerSelectionModule,
    LaunchpadMain,
};
use launchpad_guaranteed_tickets::{
    guranteed_ticket_winners::GuaranteedTicketWinnersModule, LaunchpadGuaranteedTickets,
};

pub static LAUNCHPAD_TOKEN_ID: &[u8] = b"LAUNCH-123456";
pub const LAUNCHPAD_TOKENS_PER_TICKET: u64 = 100;
pub const CONFIRM_START_EPOCH: u64 = 5;
pub const WINNER_SELECTION_START_EPOCH: u64 = 10;
pub const CLAIM_START_EPOCH: u64 = 15;

pub const NR_LAUNCHPAD_PARTICIPANTS: usize = 3;
pub const NR_WINNING_TICKETS: usize = 3;
pub const MAX_TIER_TICKETS: usize = 3;
pub const TICKET_COST: u64 = 10;

pub struct LaunchpadSetup<LaunchpadBuilder>
where
    LaunchpadBuilder: 'static + Copy + Fn() -> launchpad_guaranteed_tickets::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub participants: Vec<Address>,
    pub lp_wrapper:
        ContractObjWrapper<launchpad_guaranteed_tickets::ContractObj<DebugApi>, LaunchpadBuilder>,
}

impl<LaunchpadBuilder> LaunchpadSetup<LaunchpadBuilder>
where
    LaunchpadBuilder: 'static + Copy + Fn() -> launchpad_guaranteed_tickets::ContractObj<DebugApi>,
{
    pub fn new(lp_builder: LaunchpadBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let user_balance = rust_biguint!(TICKET_COST * MAX_TIER_TICKETS as u64);
        let total_launchpad_tokens =
            rust_biguint!(LAUNCHPAD_TOKENS_PER_TICKET * NR_WINNING_TICKETS as u64);

        let mut b_mock = BlockchainStateWrapper::new();
        let owner_address = b_mock.create_user_account(&rust_zero);
        let mut participants = Vec::new();

        for _ in 0..NR_LAUNCHPAD_PARTICIPANTS {
            let addr = b_mock.create_user_account(&user_balance);
            participants.push(addr);
        }

        b_mock.set_esdt_balance(&owner_address, LAUNCHPAD_TOKEN_ID, &total_launchpad_tokens);

        let lp_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            lp_builder,
            "buy tickets = win.wasm",
        );

        // init launchpad
        b_mock
            .execute_tx(&owner_address, &lp_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_token_id!(LAUNCHPAD_TOKEN_ID),
                    managed_biguint!(LAUNCHPAD_TOKENS_PER_TICKET),
                    EgldOrEsdtTokenIdentifier::egld(),
                    managed_biguint!(TICKET_COST),
                    NR_WINNING_TICKETS,
                    CONFIRM_START_EPOCH,
                    WINNER_SELECTION_START_EPOCH,
                    CLAIM_START_EPOCH,
                    MAX_TIER_TICKETS,
                );
            })
            .assert_ok();

        // add tickets
        // first user - 1 ticket, secnond user - 2 tickets, 3rd user - 3 tickets
        b_mock
            .execute_tx(&owner_address, &lp_wrapper, &rust_zero, |sc| {
                let mut args = MultiValueEncoded::new();
                for (i, p) in participants.iter().enumerate() {
                    args.push((managed_address!(p), i + 1).into());
                }

                sc.add_tickets_endpoint(args);

                // 1 ticket for the max tier gets removed
                assert_eq!(sc.nr_winning_tickets().get(), NR_WINNING_TICKETS - 1);
                assert_eq!(sc.max_tier_users().len(), 1);
                assert!(sc
                    .max_tier_users()
                    .contains(&managed_address!(participants.last().unwrap())));
            })
            .assert_ok();

        // deposit launchpad tokens
        b_mock
            .execute_esdt_transfer(
                &owner_address,
                &lp_wrapper,
                LAUNCHPAD_TOKEN_ID,
                0,
                &total_launchpad_tokens,
                |sc| {
                    sc.deposit_launchpad_tokens_endpoint();
                },
            )
            .assert_ok();

        b_mock.set_block_epoch(CONFIRM_START_EPOCH);

        Self {
            b_mock,
            owner_address,
            participants,
            lp_wrapper,
        }
    }

    pub fn confirm(&mut self, caller: &Address, nr_tickets: usize) -> TxResult {
        self.b_mock.execute_tx(
            caller,
            &self.lp_wrapper,
            &rust_biguint!(TICKET_COST * nr_tickets as u64),
            |sc| {
                sc.confirm_tickets(nr_tickets);
            },
        )
    }

    pub fn filter_tickets(&mut self) -> TxResult {
        self.b_mock.execute_tx(
            &self.owner_address,
            &self.lp_wrapper,
            &rust_biguint!(0),
            |sc| {
                let result = sc.filter_tickets();
                assert_eq!(result, OperationCompletionStatus::Completed);
            },
        )
    }

    pub fn select_base_winners_mock(&mut self) -> TxResult {
        self.b_mock.execute_tx(
            &self.owner_address,
            &self.lp_wrapper,
            &rust_biguint!(0),
            |sc| {
                let base_winning = NR_WINNING_TICKETS - 1;
                for ticket_id in 1..=base_winning {
                    sc.ticket_status(ticket_id).set(WINNING_TICKET);
                }

                sc.claimable_ticket_payment()
                    .set(&managed_biguint!(TICKET_COST * (base_winning as u64)));

                sc.flags().set(&Flags {
                    were_tickets_filtered: true,
                    has_winner_selection_process_started: true,
                    were_winners_selected: true,
                    was_additional_step_completed: false,
                })
            },
        )
    }

    pub fn distribute_tickets(&mut self) -> TxResult {
        self.b_mock.execute_tx(
            &self.owner_address,
            &self.lp_wrapper,
            &rust_biguint!(0),
            |sc| {
                let result = sc.distribute_guaranteed_tickets();
                assert_eq!(result, OperationCompletionStatus::Completed);
            },
        )
    }

    pub fn claim_user(&mut self, user: &Address) -> TxResult {
        self.b_mock
            .execute_tx(user, &self.lp_wrapper, &rust_biguint!(0), |sc| {
                sc.claim_launchpad_tokens();
            })
    }

    pub fn claim_owner(&mut self) -> TxResult {
        self.b_mock.execute_tx(
            &self.owner_address,
            &self.lp_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.claim_ticket_payment();
            },
        )
    }
}
