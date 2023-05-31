#![allow(clippy::bool_assert_comparison)]

use combined_selection_setup::{
    LaunchpadSetup, BASE_TICKET_COST, CLAIM_START_BLOCK, LAUNCHPAD_TOKENS_PER_TICKET,
    LAUNCHPAD_TOKEN_ID, NFT_TICKET_COST, SFT_TOKEN_ID, WINNER_SELECTION_START_BLOCK,
};
use launchpad_common::{
    config::ConfigModule,
    tickets::{TicketsModule, WINNING_TICKET},
};
use launchpad_guaranteed_tickets::guaranteed_tickets_init::GuaranteedTicketsInitModule;
use launchpad_with_nft::{
    confirm_nft::ConfirmNftModule, mystery_sft::MysterySftTypes,
    nft_winners_selection::NftWinnersSelectionModule,
};
use multiversx_sc::codec::Empty;
use multiversx_sc_scenario::{managed_address, managed_biguint, rust_biguint};

use crate::combined_selection_setup::{MAX_TIER_TICKETS, NR_WINNING_TICKETS, TOTAL_NFTS};

pub mod combined_selection_setup;

#[test]
fn setup_test() {
    let mut lp_setup = LaunchpadSetup::new(launchpad_nft_and_guaranteed_tickets::contract_obj);
    let part = lp_setup.participants.clone();

    lp_setup
        .b_mock
        .execute_query(&lp_setup.lp_wrapper, |sc| {
            assert_eq!(sc.nr_winning_tickets().get(), NR_WINNING_TICKETS - 1);
            assert_eq!(
                sc.min_confirmed_for_guaranteed_ticket().get(),
                MAX_TIER_TICKETS
            );
            assert_eq!(sc.users_with_guaranteed_ticket().len(), 1);
            assert!(sc
                .users_with_guaranteed_ticket()
                .contains(&managed_address!(part.last().unwrap())));

            assert_eq!(sc.total_available_nfts().get(), TOTAL_NFTS);
            assert_eq!(sc.confirmed_nft_user_list().len(), 2);
            assert!(
                sc.confirmed_nft_user_list()
                    .contains(&managed_address!(&part[0]))
                    && sc
                        .confirmed_nft_user_list()
                        .contains(&managed_address!(&part[1]))
            );

            // only claimable after selected winners
            assert_eq!(sc.claimable_ticket_payment().get(), managed_biguint!(0));
            assert_eq!(sc.claimable_nft_payment().get(), managed_biguint!(0));
        })
        .assert_ok();

    let initial_user_balance =
        rust_biguint!(BASE_TICKET_COST * MAX_TIER_TICKETS as u64 + NFT_TICKET_COST);
    let mut sc_balance = rust_biguint!(0);
    for (i, p) in part.iter().enumerate() {
        let mut user_payment = &rust_biguint!(NFT_TICKET_COST) + (i + 1) as u64 * BASE_TICKET_COST;

        // last user didn't confirm NFT
        if i == part.len() - 1 {
            user_payment -= NFT_TICKET_COST;
        }

        sc_balance += &user_payment;

        let expected_balance = &initial_user_balance - &user_payment;
        lp_setup.b_mock.check_egld_balance(p, &expected_balance);
    }

    lp_setup
        .b_mock
        .check_egld_balance(lp_setup.lp_wrapper.address_ref(), &sc_balance);
}

#[test]
fn combined_selection_test() {
    let mut lp_setup = LaunchpadSetup::new(launchpad_nft_and_guaranteed_tickets::contract_obj);
    let part = lp_setup.participants.clone();

    lp_setup
        .b_mock
        .set_block_nonce(WINNER_SELECTION_START_BLOCK);
    lp_setup.select_base_launchpad_winners().assert_ok();

    lp_setup
        .b_mock
        .execute_query(&lp_setup.lp_wrapper, |sc| {
            assert_eq!(sc.ticket_status(1).get(), WINNING_TICKET);
            assert_eq!(sc.ticket_status(2).get(), false);
            assert_eq!(sc.ticket_status(3).get(), false);
            assert_eq!(sc.ticket_status(4).get(), false);
            assert_eq!(sc.ticket_status(5).get(), false);
            assert_eq!(sc.ticket_status(6).get(), false);

            assert_eq!(sc.nr_winning_tickets().get(), NR_WINNING_TICKETS - 1);

            assert_eq!(sc.confirmed_nft_user_list().len(), 2);
            assert_eq!(sc.nft_selection_winners().len(), 0);

            assert_eq!(
                sc.claimable_ticket_payment().get(),
                managed_biguint!(BASE_TICKET_COST * (NR_WINNING_TICKETS - 1) as u64)
            );
            assert_eq!(sc.claimable_nft_payment().get(), managed_biguint!(0));
        })
        .assert_ok();

    lp_setup.secondary_selection_step_single_call().assert_ok();

    lp_setup
        .b_mock
        .execute_query(&lp_setup.lp_wrapper, |sc| {
            assert_eq!(sc.ticket_status(1).get(), WINNING_TICKET);
            assert_eq!(sc.ticket_status(2).get(), false);
            assert_eq!(sc.ticket_status(3).get(), false);
            assert_eq!(sc.ticket_status(4).get(), WINNING_TICKET);
            assert_eq!(sc.ticket_status(5).get(), false);
            assert_eq!(sc.ticket_status(6).get(), false);

            assert_eq!(sc.nr_winning_tickets().get(), NR_WINNING_TICKETS);

            assert_eq!(sc.confirmed_nft_user_list().len(), 1);
            assert_eq!(sc.nft_selection_winners().len(), 1);

            assert!(sc
                .confirmed_nft_user_list()
                .contains(&managed_address!(&part[0])));
            assert!(sc
                .nft_selection_winners()
                .contains(&managed_address!(&part[1])));

            assert_eq!(
                sc.claimable_ticket_payment().get(),
                managed_biguint!(BASE_TICKET_COST * NR_WINNING_TICKETS as u64)
            );
            assert_eq!(
                sc.claimable_nft_payment().get(),
                managed_biguint!(NFT_TICKET_COST * TOTAL_NFTS as u64)
            );
        })
        .assert_ok();

    lp_setup.b_mock.set_block_nonce(CLAIM_START_BLOCK);

    for p in &part {
        lp_setup.claim(p).assert_ok();
    }
    lp_setup.owner_claim().assert_ok();

    // initial user balance = 130, 10 cost per ticket, 100 cost per NFT ticket
    // user[0] won 1 ticket
    // user[1] won 0 tickets, but won NFT
    // user[2] won 1 ticket (guaranteed)

    // check EGLD balance
    lp_setup
        .b_mock
        .check_egld_balance(&part[0], &rust_biguint!(120));
    lp_setup
        .b_mock
        .check_egld_balance(&part[1], &rust_biguint!(30));
    lp_setup
        .b_mock
        .check_egld_balance(&part[2], &rust_biguint!(120));

    // payment for 2 tickets and an NFT
    lp_setup
        .b_mock
        .check_egld_balance(&lp_setup.owner_address, &rust_biguint!(120));
    lp_setup
        .b_mock
        .check_egld_balance(lp_setup.lp_wrapper.address_ref(), &rust_biguint!(0));

    // check launchpad tokens balance
    lp_setup.b_mock.check_esdt_balance(
        &part[0],
        LAUNCHPAD_TOKEN_ID,
        &rust_biguint!(LAUNCHPAD_TOKENS_PER_TICKET),
    );
    lp_setup
        .b_mock
        .check_esdt_balance(&part[1], LAUNCHPAD_TOKEN_ID, &rust_biguint!(0));
    lp_setup.b_mock.check_esdt_balance(
        &part[2],
        LAUNCHPAD_TOKEN_ID,
        &rust_biguint!(LAUNCHPAD_TOKENS_PER_TICKET),
    );

    // check received SFTs
    lp_setup.b_mock.check_nft_balance(
        &part[0],
        SFT_TOKEN_ID,
        MysterySftTypes::ConfirmedLost.as_nonce(),
        &rust_biguint!(1),
        Some(&Empty),
    );
    lp_setup.b_mock.check_nft_balance(
        &part[1],
        SFT_TOKEN_ID,
        MysterySftTypes::ConfirmedWon.as_nonce(),
        &rust_biguint!(1),
        Some(&Empty),
    );
    lp_setup.b_mock.check_nft_balance(
        &part[2],
        SFT_TOKEN_ID,
        MysterySftTypes::NotConfirmed.as_nonce(),
        &rust_biguint!(1),
        Some(&Empty),
    );
}
