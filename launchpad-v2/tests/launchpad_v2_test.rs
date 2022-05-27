mod launchpad_v2_setup;

use elrond_wasm_debug::{managed_address, rust_biguint};
use launchpad_common::tickets::{TicketsModule, WINNING_TICKET};
use launchpad_v2::{
    confirm_nft::ConfirmNftModule, nft_winners_selection::NftWinnersSelectionModule,
};
use launchpad_v2_setup::*;

#[test]
fn init_test() {
    let _ = LaunchpadSetup::new(launchpad_v2::contract_obj);
}

#[test]
fn confirm_test() {
    let mut lp_setup = LaunchpadSetup::new(launchpad_v2::contract_obj);

    // confirm ok
    let users = lp_setup.participants.clone();
    for user in &users {
        lp_setup.confirm_nft(user).assert_ok();
    }

    lp_setup
        .b_mock
        .execute_query(&lp_setup.lp_wrapper, |sc| {
            let user_mapper = sc.confirmed_nft_user_list();
            assert_eq!(user_mapper.len(), NR_LAUNCHPAD_PARTICIPANTS);

            for user in &users {
                user_mapper.contains(&managed_address!(user));
            }
        })
        .assert_ok();

    // try confirm again
    lp_setup
        .b_mock
        .set_egld_balance(&users[0], &rust_biguint!(1_000_000_000));
    lp_setup
        .confirm_nft(&users[0])
        .assert_user_error("Already confirmed NFT");
}

#[test]
fn select_winners_test() {
    let mut lp_setup = LaunchpadSetup::new(launchpad_v2::contract_obj);
    let users = lp_setup.participants.clone();

    lp_setup.confirm_nft(&users[0]).assert_ok();
    lp_setup.confirm_nft(&users[1]).assert_ok();

    lp_setup
        .b_mock
        .set_block_epoch(WINNER_SELECTION_START_EPOCH);

    // try select nft winners before base launchpad
    lp_setup
        .select_nft_winners()
        .assert_user_error("Must select winners for base launchpad first");

    // select base launchpad winners
    lp_setup.select_base_launchpad_winners().assert_ok();

    // ticket #1 won
    lp_setup
        .b_mock
        .execute_query(&lp_setup.lp_wrapper, |sc| {
            assert_eq!(sc.ticket_status(1).get(), WINNING_TICKET);
            assert_eq!(sc.ticket_status(2).get(), false);
            assert_eq!(sc.ticket_status(3).get(), false);
        })
        .assert_ok();

    // select nft winners
    lp_setup.select_nft_winners().assert_ok();

    // user[0] won
    lp_setup
        .b_mock
        .execute_query(&lp_setup.lp_wrapper, |sc| {
            assert_eq!(sc.confirmed_nft_user_list().len(), 1);
            assert!(sc
                .confirmed_nft_user_list()
                .contains(&managed_address!(&users[1])));

            assert_eq!(sc.nft_selection_winners().len(), 1);
            assert!(sc
                .nft_selection_winners()
                .contains(&managed_address!(&users[0])));
        })
        .assert_ok();
}

#[test]
fn claim_test() {
    let mut lp_setup = LaunchpadSetup::new(launchpad_v2::contract_obj);
    let users = lp_setup.participants.clone();

    lp_setup.confirm_nft(&users[0]).assert_ok();
    lp_setup.confirm_nft(&users[1]).assert_ok();

    lp_setup
        .b_mock
        .set_block_epoch(WINNER_SELECTION_START_EPOCH);

    lp_setup.select_base_launchpad_winners().assert_ok();
    lp_setup.select_nft_winners().assert_ok();

    lp_setup.b_mock.set_block_epoch(CLAIM_START_EPOCH);

    for user in &users {
        lp_setup.claim(user).assert_ok();
    }
}
