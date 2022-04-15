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
        getConfirmationPeriodStartEpoch
        getLaunchpadTokenId
        getLaunchpadTokensPerWinningTicket
        getNumberOfConfirmedTicketsForAddress
        getNumberOfWinningTickets
        getNumberOfWinningTicketsForAddress
        getTicketPaymentToken
        getTicketPrice
        getTicketRangeForAddress
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
        setTicketPaymentToken
        setTicketPrice
        setWinnerSelectionStartEpoch
        wereWinnersSelected
    )
}

elrond_wasm_node::wasm_empty_callback! {}
