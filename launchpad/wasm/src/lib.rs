////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    launchpad
    (
        addTickets
        addUsersToBlacklist
        claimLaunchpadTokens
        claimTicketPayment
        confirmTickets
        depositLaunchpadTokens
        filterTickets
        getConfiguration
        getLaunchStageFlags
        getLaunchpadTokenId
        getLaunchpadTokensPerWinningTicket
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
