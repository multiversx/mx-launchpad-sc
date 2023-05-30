mod launchpad_with_nft_setup;

use launchpad_common::tickets::{TicketsModule, WINNING_TICKET};
use launchpad_with_nft::{
    confirm_nft::ConfirmNftModule, mystery_sft::MysterySftTypes,
    nft_winners_selection::NftWinnersSelectionModule, Launchpad,
};
use launchpad_with_nft_setup::*;
use multiversx_sc::{codec::Empty, types::MultiValueEncoded};
use multiversx_sc_scenario::{managed_address, managed_biguint, rust_biguint};

#[test]
fn init_test() {
    let _ = LaunchpadSetup::new(launchpad_with_nft::contract_obj);
}

#[test]
fn confirm_test() {
    let mut lp_setup = LaunchpadSetup::new(launchpad_with_nft::contract_obj);

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
    let mut lp_setup = LaunchpadSetup::new(launchpad_with_nft::contract_obj);
    let users = lp_setup.participants.clone();

    lp_setup.confirm_nft(&users[0]).assert_ok();
    lp_setup.confirm_nft(&users[1]).assert_ok();

    lp_setup
        .b_mock
        .set_block_nonce(WINNER_SELECTION_START_BLOCK);

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
    let mut lp_setup = LaunchpadSetup::new(launchpad_with_nft::contract_obj);
    let users = lp_setup.participants.clone();

    lp_setup.confirm_nft(&users[0]).assert_ok();
    lp_setup.confirm_nft(&users[1]).assert_ok();

    lp_setup
        .b_mock
        .set_block_nonce(WINNER_SELECTION_START_BLOCK);

    lp_setup.select_base_launchpad_winners().assert_ok();
    lp_setup.select_nft_winners().assert_ok();

    lp_setup.b_mock.set_block_nonce(CLAIM_START_BLOCK);

    for user in &users {
        lp_setup.claim(user).assert_ok();
    }

    // check NFT balances
    lp_setup.b_mock.check_nft_balance(
        &users[0],
        SFT_TOKEN_ID,
        MysterySftTypes::ConfirmedWon.as_nonce(),
        &rust_biguint!(1u32),
        Some(&Empty),
    );
    lp_setup.b_mock.check_nft_balance(
        &users[1],
        SFT_TOKEN_ID,
        MysterySftTypes::ConfirmedLost.as_nonce(),
        &rust_biguint!(1u32),
        Some(&Empty),
    );
    lp_setup.b_mock.check_nft_balance(
        &users[2],
        SFT_TOKEN_ID,
        MysterySftTypes::NotConfirmed.as_nonce(),
        &rust_biguint!(1u32),
        Some(&Empty),
    );

    // check EGLD balances
    let initial_balance = BASE_TICKET_COST + NFT_TICKET_COST;
    lp_setup
        .b_mock
        .check_egld_balance(&users[0], &rust_biguint!(0));
    lp_setup
        .b_mock
        .check_egld_balance(&users[1], &rust_biguint!(initial_balance));
    lp_setup
        .b_mock
        .check_egld_balance(&users[2], &rust_biguint!(initial_balance));

    lp_setup.b_mock.check_egld_balance(
        &lp_setup.lp_wrapper.address_ref(),
        &rust_biguint!(BASE_TICKET_COST + NFT_TICKET_COST),
    );
    lp_setup
        .b_mock
        .check_egld_balance(&lp_setup.owner_address, &rust_biguint!(0));

    // owner claim nft ticket payment
    lp_setup
        .b_mock
        .execute_query(&lp_setup.lp_wrapper, |sc| {
            assert_eq!(
                sc.claimable_nft_payment().get(),
                managed_biguint!(NFT_TICKET_COST),
            );
        })
        .assert_ok();

    lp_setup
        .b_mock
        .execute_tx(
            &lp_setup.owner_address,
            &lp_setup.lp_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.claim_nft_payment();
            },
        )
        .assert_ok();

    lp_setup.b_mock.check_egld_balance(
        &lp_setup.lp_wrapper.address_ref(),
        &rust_biguint!(BASE_TICKET_COST),
    );
    lp_setup
        .b_mock
        .check_egld_balance(&lp_setup.owner_address, &rust_biguint!(NFT_TICKET_COST));

    lp_setup
        .b_mock
        .execute_query(&lp_setup.lp_wrapper, |sc| {
            assert!(sc.claimable_nft_payment().is_empty());
        })
        .assert_ok();
}

#[test]
fn blacklist_refund_test() {
    let mut lp_setup = LaunchpadSetup::new(launchpad_with_nft::contract_obj);

    // confirm ok
    let users = lp_setup.participants.clone();
    for user in &users {
        lp_setup.confirm_nft(user).assert_ok();
    }

    lp_setup
        .b_mock
        .execute_tx(
            &lp_setup.owner_address,
            &lp_setup.lp_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut args = MultiValueEncoded::new();
                args.push(managed_address!(&users[0]));

                sc.add_users_to_blacklist_endpoint(args);
            },
        )
        .assert_ok();

    lp_setup.b_mock.check_egld_balance(
        &users[0],
        &rust_biguint!(BASE_TICKET_COST + NFT_TICKET_COST),
    );
}
