// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                           43
// Async Callback (empty):               1
// Total number of exported functions:  46

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    launchpad_guaranteed_tickets_v2
    (
        init => init
        upgrade => upgrade
        addTickets => add_tickets_endpoint
        depositLaunchpadTokens => deposit_launchpad_tokens_endpoint
        refundUserTickets => refund_user_tickets
        addUsersToBlacklist => add_users_to_blacklist_endpoint
        removeGuaranteedUsersFromBlacklist => remove_guaranteed_users_from_blacklist_endpoint
        distributeGuaranteedTickets => distribute_guaranteed_tickets_endpoint
        claimLaunchpadTokens => claim_launchpad_tokens_endpoint
        claimTicketPayment => claim_ticket_payment_endpoint
        getUserTicketsStatus => user_tickets_status
        getLaunchStageFlags => flags
        getConfiguration => configuration
        getLaunchpadTokenId => launchpad_token_id
        getLaunchpadTokensPerWinningTicket => launchpad_tokens_per_winning_ticket
        getTicketPrice => ticket_price
        getNumberOfWinningTickets => nr_winning_tickets
        getTotalLaunchpadTokensDeposited => total_launchpad_tokens_deposited
        setTicketPrice => set_ticket_price
        setLaunchpadTokensPerWinningTicket => set_launchpad_tokens_per_winning_ticket
        setConfirmationPeriodStartRound => set_confirmation_period_start_round
        setWinnerSelectionStartRound => set_winner_selection_start_round
        setClaimStartRound => set_claim_start_round
        getTicketRangeForAddress => get_ticket_range_for_address
        getTotalNumberOfTicketsForAddress => get_total_number_of_tickets_for_address
        getTotalNumberOfTickets => last_ticket_id
        getNumberOfConfirmedTicketsForAddress => nr_confirmed_tickets
        filterTickets => filter_tickets
        selectWinners => select_winners
        getNumberOfWinningTicketsForAddress => get_number_of_winning_tickets_for_address
        getWinningTicketIdsForAddress => get_winning_ticket_ids_for_address
        setSupportAddress => add_support_address
        getSupportAddress => support_address
        isUserBlacklisted => is_user_blacklisted
        confirmTickets => confirm_tickets
        hasUserClaimedTokens => has_user_claimed
        setUnlockSchedule => set_unlock_schedule
        refundWinningTickets => refund_winning_tickets
        getClaimableTokens => compute_claimable_tokens
        getUserTotalClaimableBalance => user_total_claimable_balance
        getUserClaimedBalance => user_claimed_balance
        getUnlockSchedule => unlock_schedule
        pause => pause_endpoint
        unpause => unpause_endpoint
        isPaused => paused_status
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
