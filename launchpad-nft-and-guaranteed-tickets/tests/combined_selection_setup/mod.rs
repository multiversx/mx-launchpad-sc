use launchpad_common::{
    user_interactions::UserInteractionsModule, winner_selection::WinnerSelectionModule,
};
use launchpad_nft_and_guaranteed_tickets::{
    combined_selection::CombinedSelectionModule, Launchpad,
};
use launchpad_with_nft::{
    confirm_nft::ConfirmNftModule,
    mystery_sft::{MysterySftModule, SftSetupSteps},
};
use multiversx_sc::{
    storage::mappers::StorageTokenWrapper,
    types::{
        Address, EgldOrEsdtTokenIdentifier, EsdtLocalRole, MultiValueEncoded,
        OperationCompletionStatus,
    },
};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper, TxResult},
    DebugApi,
};

pub const NR_LAUNCHPAD_PARTICIPANTS: usize = 3;
pub const BASE_TICKET_COST: u64 = 10;
pub const NFT_TICKET_COST: u64 = 100;

pub static LAUNCHPAD_TOKEN_ID: &[u8] = b"LAUNCH-123456";
pub const LAUNCHPAD_TOKENS_PER_TICKET: u64 = 100;
pub const NR_WINNING_TICKETS: usize = 2;
pub const MAX_TIER_TICKETS: usize = 3;
pub const TOTAL_NFTS: usize = 1;

pub const CONFIRM_START_ROUND: u64 = 5;
pub const WINNER_SELECTION_START_ROUND: u64 = 10;
pub const CLAIM_START_ROUND: u64 = 15;

pub static SFT_TOKEN_ID: &[u8] = b"MYSTERY-123456";

pub struct LaunchpadSetup<LaunchpadBuilder>
where
    LaunchpadBuilder:
        'static + Copy + Fn() -> launchpad_nft_and_guaranteed_tickets::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub participants: Vec<Address>,
    pub lp_wrapper: ContractObjWrapper<
        launchpad_nft_and_guaranteed_tickets::ContractObj<DebugApi>,
        LaunchpadBuilder,
    >,
}

impl<LaunchpadBuilder> LaunchpadSetup<LaunchpadBuilder>
where
    LaunchpadBuilder:
        'static + Copy + Fn() -> launchpad_nft_and_guaranteed_tickets::ContractObj<DebugApi>,
{
    pub fn new(lp_builder: LaunchpadBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let user_balance =
            rust_biguint!(BASE_TICKET_COST * MAX_TIER_TICKETS as u64 + NFT_TICKET_COST);
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
            "launchpad_with_nft.wasm",
        );

        // init launchpad
        b_mock
            .execute_tx(&owner_address, &lp_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_token_id!(LAUNCHPAD_TOKEN_ID),
                    managed_biguint!(LAUNCHPAD_TOKENS_PER_TICKET),
                    EgldOrEsdtTokenIdentifier::egld(),
                    managed_biguint!(BASE_TICKET_COST),
                    NR_WINNING_TICKETS,
                    CONFIRM_START_ROUND,
                    WINNER_SELECTION_START_ROUND,
                    CLAIM_START_ROUND,
                    EgldOrEsdtTokenIdentifier::egld(),
                    0,
                    managed_biguint!(NFT_TICKET_COST),
                    TOTAL_NFTS,
                    MAX_TIER_TICKETS,
                );
            })
            .assert_ok();

        // setup mystery sft
        b_mock.set_esdt_local_roles(
            lp_wrapper.address_ref(),
            SFT_TOKEN_ID,
            &[
                EsdtLocalRole::NftCreate,
                EsdtLocalRole::NftAddQuantity,
                EsdtLocalRole::NftBurn,
            ],
        );

        b_mock
            .execute_tx(&owner_address, &lp_wrapper, &rust_zero, |sc| {
                sc.mystery_sft()
                    .set_token_id(managed_token_id!(SFT_TOKEN_ID));
                sc.create_initial_sfts();
                sc.sft_setup_steps().set(&SftSetupSteps {
                    issued_token: true,
                    created_initial_tokens: true,
                    set_transfer_role: true,
                });
            })
            .assert_ok();

        // add tickets
        b_mock
            .execute_tx(&owner_address, &lp_wrapper, &rust_zero, |sc| {
                let mut args = MultiValueEncoded::new();
                for (i, p) in participants.iter().enumerate() {
                    args.push((managed_address!(p), i + 1, 0, false).into());
                }

                sc.add_tickets_endpoint(args);
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

        b_mock.set_block_round(CONFIRM_START_ROUND);

        // confirm base launchpad tickets
        for (i, p) in participants.iter().enumerate() {
            let nr_tickets = i + 1;
            b_mock
                .execute_tx(
                    p,
                    &lp_wrapper,
                    &rust_biguint!(nr_tickets as u64 * BASE_TICKET_COST),
                    |sc| {
                        sc.confirm_tickets(nr_tickets);
                    },
                )
                .assert_ok();
        }

        // confirm nft
        for p in &participants[0..participants.len() - 1] {
            b_mock
                .execute_tx(p, &lp_wrapper, &rust_biguint!(NFT_TICKET_COST), |sc| {
                    sc.confirm_nft();
                })
                .assert_ok();
        }

        Self {
            b_mock,
            owner_address,
            participants,
            lp_wrapper,
        }
    }

    pub fn confirm_nft(&mut self, caller: &Address) -> TxResult {
        self.b_mock.execute_tx(
            caller,
            &self.lp_wrapper,
            &rust_biguint!(NFT_TICKET_COST),
            |sc| {
                sc.confirm_nft();
            },
        )
    }

    pub fn select_base_launchpad_winners(&mut self) -> TxResult {
        self.b_mock
            .execute_tx(
                &self.owner_address,
                &self.lp_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let result = sc.filter_tickets();
                    assert!(matches!(result, OperationCompletionStatus::Completed));
                },
            )
            .assert_ok();

        self.b_mock.execute_tx(
            &self.owner_address,
            &self.lp_wrapper,
            &rust_biguint!(0),
            |sc| {
                let result = sc.select_winners();
                assert!(matches!(result, OperationCompletionStatus::Completed));
            },
        )
    }

    pub fn secondary_selection_step_single_call(&mut self) -> TxResult {
        self.b_mock.execute_tx(
            &self.owner_address,
            &self.lp_wrapper,
            &rust_biguint!(0),
            |sc| {
                let result = sc.secondary_selection_step();
                assert!(matches!(result, OperationCompletionStatus::Completed));
            },
        )
    }

    pub fn claim(&mut self, caller: &Address) -> TxResult {
        self.b_mock
            .execute_tx(caller, &self.lp_wrapper, &rust_biguint!(0), |sc| {
                sc.claim_launchpad_tokens_endpoint();
            })
    }

    pub fn owner_claim(&mut self) -> TxResult {
        self.b_mock.execute_tx(
            &self.owner_address,
            &self.lp_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.claim_ticket_payment_endpoint();
            },
        )
    }
}
