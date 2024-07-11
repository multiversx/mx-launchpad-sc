multiversx_sc::derive_imports!();

use launchpad_common::{
    config::ConfigModule, user_interactions::UserInteractionsModule,
    winner_selection::WinnerSelectionModule,
};
use launchpad_locked_tokens::LaunchpadLockedTokens;
use multiversx_sc::{
    api::ManagedTypeApi,
    codec::{TopDecode, TopEncode},
    contract_base::{CallableContract, ContractBase},
    types::{
        EgldOrEsdtTokenIdentifier, EsdtLocalRole, EsdtTokenPayment, ManagedAddress,
        MultiValueEncoded,
    },
};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_egld_token_id, managed_token_id,
    managed_token_id_wrapped, rust_biguint,
    testing_framework::{BlockchainStateWrapper, TxContextStack},
    DebugApi,
};

static LOCK_FN_NAME: &str = "lockTokens";
static LOCKED_TOKEN_ID: &[u8] = b"LKTOK-123456";
static LAUNCHPAD_TOKEN_ID: &[u8] = b"LAUNCH-123456";
const LAUNCHPAD_TOKENS_PER_TICKET: u64 = 100_000;
const TICKET_PRICE: u64 = 100;
const NR_WINNING_TICKETS: usize = 1;
const CONFIRM_START_BLOCK: u64 = 10;
const WINNER_SELECTION_START_BLOCK: u64 = 20;
const CLAIM_START_BLOCK: u64 = 30;
const LOCK_PERCENTAGE: u32 = 5_000; // 50%
const UNLOCK_EPOCH: u64 = 10;

#[test]
fn launchpad_with_locked_tokens_out_test() {
    let _ = DebugApi::dummy();
    let mut b_mock = BlockchainStateWrapper::new();
    let rust_zero = rust_biguint!(0);

    let owner = b_mock.create_user_account(&rust_zero);
    let user = b_mock.create_user_account(&rust_biguint!(TICKET_PRICE));
    let simple_lock_sc =
        b_mock.create_sc_account(&rust_zero, None, SimpleLockMock::new, "simple lock wasm");
    let lp_sc = b_mock.create_sc_account(
        &rust_zero,
        Some(&owner),
        launchpad_locked_tokens::contract_obj,
        "launchpad wasm",
    );

    // setup
    b_mock
        .execute_tx(&owner, &lp_sc, &rust_zero, |sc| {
            sc.init(
                managed_token_id!(LAUNCHPAD_TOKEN_ID),
                managed_biguint!(LAUNCHPAD_TOKENS_PER_TICKET),
                managed_egld_token_id!(),
                managed_biguint!(TICKET_PRICE),
                NR_WINNING_TICKETS,
                CONFIRM_START_BLOCK,
                WINNER_SELECTION_START_BLOCK,
                CLAIM_START_BLOCK,
                LOCK_PERCENTAGE,
                UNLOCK_EPOCH,
                managed_address!(simple_lock_sc.address_ref()),
            );

            let mut tickets = MultiValueEncoded::new();
            tickets.push((managed_address!(&user), 1).into());
            sc.add_tickets_endpoint(tickets);

            sc.launchpad_tokens_deposited().set(true);
        })
        .assert_ok();

    b_mock.set_esdt_balance(
        lp_sc.address_ref(),
        LAUNCHPAD_TOKEN_ID,
        &rust_biguint!(NR_WINNING_TICKETS as u64 * LAUNCHPAD_TOKENS_PER_TICKET),
    );

    b_mock.set_esdt_local_roles(
        simple_lock_sc.address_ref(),
        LOCKED_TOKEN_ID,
        &[EsdtLocalRole::NftCreate],
    );

    // user confirm
    b_mock.set_block_nonce(CONFIRM_START_BLOCK);

    b_mock
        .execute_tx(&user, &lp_sc, &rust_biguint!(TICKET_PRICE), |sc| {
            sc.confirm_tickets(1);
        })
        .assert_ok();

    // filter + select winners
    b_mock.set_block_nonce(WINNER_SELECTION_START_BLOCK);

    b_mock
        .execute_tx(&owner, &lp_sc, &rust_zero, |sc| {
            sc.filter_tickets();
            sc.select_winners();
        })
        .assert_ok();

    // user claim
    b_mock.set_block_nonce(CLAIM_START_BLOCK);

    b_mock
        .execute_tx(&user, &lp_sc, &rust_zero, |sc| {
            sc.claim_launchpad_tokens_endpoint();
        })
        .assert_ok();

    // check balance
    b_mock.check_esdt_balance(
        &user,
        LAUNCHPAD_TOKEN_ID,
        &rust_biguint!(LAUNCHPAD_TOKENS_PER_TICKET / 2),
    );

    b_mock.check_nft_balance(
        &user,
        LOCKED_TOKEN_ID,
        1,
        &rust_biguint!(LAUNCHPAD_TOKENS_PER_TICKET / 2),
        Some(&LockedTokenAttributes::<DebugApi> {
            original_token_id: managed_token_id_wrapped!(LAUNCHPAD_TOKEN_ID),
            original_token_nonce: 0,
            unlock_epoch: UNLOCK_EPOCH,
        }),
    );
}

#[derive(Clone, Default)]
pub struct SimpleLockMock {}

impl ContractBase for SimpleLockMock {
    type Api = DebugApi;
}

impl CallableContract for SimpleLockMock {
    fn call(&self, fn_name: &str) -> bool {
        if fn_name != LOCK_FN_NAME {
            return false;
        }

        self.call_lock_tokens();

        true
    }
}

impl SimpleLockMock {
    pub fn new() -> Self {
        SimpleLockMock {}
    }

    fn call_lock_tokens(&self) {
        let api = TxContextStack::static_peek();
        let args = api.input_ref().args.clone();
        if args.len() != 2 {
            panic!("Invalid args");
        }

        // drop(api);

        let unlock_epoch = u64::top_decode(args[0].clone()).unwrap();
        let dest_addr = ManagedAddress::<DebugApi>::top_decode(args[1].clone()).unwrap();

        let payment = self.call_value().egld_or_single_esdt();
        let current_epoch = self.blockchain().get_block_epoch();
        if current_epoch >= unlock_epoch {
            self.send().direct(
                &dest_addr,
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            );

            let mut result = Vec::new();
            payment.top_encode(&mut result).unwrap();
            api.tx_result_cell
                .try_lock()
                .unwrap()
                .result_values
                .push(result);

            return;
        }

        let attributes = LockedTokenAttributes {
            original_token_id: payment.token_identifier.clone(),
            original_token_nonce: payment.token_nonce,
            unlock_epoch,
        };
        let locked_token_nonce = self.send().esdt_nft_create_compact_named(
            &managed_token_id!(LOCKED_TOKEN_ID),
            &payment.amount,
            &payment.token_identifier.clone().into_name(),
            &attributes,
        );
        self.send().direct_esdt(
            &dest_addr,
            &managed_token_id!(LOCKED_TOKEN_ID),
            locked_token_nonce,
            &payment.amount,
        );

        let output_payment = EsdtTokenPayment::new(
            managed_token_id!(LOCKED_TOKEN_ID),
            locked_token_nonce,
            payment.amount,
        );
        let mut result = Vec::new();
        output_payment.top_encode(&mut result).unwrap();
        api.tx_result_cell
            .try_lock()
            .unwrap()
            .result_values
            .push(result);
    }
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedDecode, NestedEncode, PartialEq, Debug)]
pub struct LockedTokenAttributes<M: ManagedTypeApi> {
    pub original_token_id: EgldOrEsdtTokenIdentifier<M>,
    pub original_token_nonce: u64,
    pub unlock_epoch: u64,
}
