////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    launchpad_locked_tokens_and_guaranteed_tickets
    (
        addTickets
        addUsersToBlacklist
        claimLaunchpadTokens
        claimTicketPayment
        confirmTickets
        depositLaunchpadTokens
        distributeGuaranteedTickets
        filterTickets
        getConfiguration
        getLaunchStageFlags
        getLaunchpadTokenId
        getLaunchpadTokensLockPercentage
        getLaunchpadTokensPerWinningTicket
        getLaunchpadTokensUnlockEpoch
        getNumberOfConfirmedTicketsForAddress
        getNumberOfWinningTickets
        getNumberOfWinningTicketsForAddress
        getSupportAddress
        getTicketPrice
        getTicketRangeForAddress
        getTotalNumberOfTickets
        getTotalNumberOfTicketsForAddress
        getWinningTicketIdsForAddress
        hasUserClaimedTokens
        isUserBlacklisted
        removeUsersFromBlacklist
        selectWinners
        setClaimStartBlock
        setConfirmationPeriodStartBlock
        setLaunchpadTokensPerWinningTicket
        setSupportAddress
        setTicketPrice
        setWinnerSelectionStartBlock
    )
}

elrond_wasm_node::wasm_empty_callback! {}
