////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    launchpad_nft_and_guaranteed_tickets
    (
        callBack
        addTickets
        addUsersToBlacklist
        claimLaunchpadTokens
        claimTicketPayment
        confirmNft
        confirmTickets
        createInitialSfts
        depositLaunchpadTokens
        filterTickets
        getConfiguration
        getLaunchStageFlags
        getLaunchpadTokenId
        getLaunchpadTokensPerWinningTicket
        getNftCost
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
        hasUserConfirmedNft
        hasUserWonNft
        isUserBlacklisted
        issueMysterySft
        removeUsersFromBlacklist
        secondarySelectionStep
        selectWinners
        setClaimStartEpoch
        setConfirmationPeriodStartEpoch
        setLaunchpadTokensPerWinningTicket
        setSupportAddress
        setTicketPrice
        setTransferRole
        setWinnerSelectionStartEpoch
        unsetTransferRole
    )
}
