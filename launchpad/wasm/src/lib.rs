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
        getClaimStartEpoch
        getConfiguration
        getConfirmationPeriodStartEpoch
        getLaunchStageFlags
        getLaunchpadTokenId
        getLaunchpadTokensPerWinningTicket
        getNumberOfConfirmedTicketsForAddress
        getNumberOfWinningTickets
        getNumberOfWinningTicketsForAddress
        getSupportAddress
        getTicketPaymentToken
        getTicketPrice
        getTicketRangeForAddress
        getTotalNumberOfTickets
        getTotalNumberOfTicketsForAddress
        getWinnerSelectionStart
        getWinningTicketIdsForAddress
        hasUserClaimedTokens
        isUserBlacklisted
        removeUsersFromBlacklist
        selectWinners
        setClaimStartEpoch
        setConfirmationPeriodStartEpoch
        setLaunchpadTokensPerWinningTicket
        setSupportAddress
        setTicketPaymentToken
        setTicketPrice
        setWinnerSelectionStartEpoch
        wereWinnersSelected
    )
}

elrond_wasm_node::wasm_empty_callback! {}
