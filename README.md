# sc-launchpad-rs

## Launchpad smart contract

On the Elrond Protocol we aim to start several “launchpad” projects and for the token launch we want to create a set of smart contracts which handle in a decentralized manner the selection of token buyers. The basic idea is that the user is eligible to buy a set of lottery tickets for the given launchpad according to the number of EGLD tokens the user stakes / delegates. On top of this every user who wants to participate in the token sale event must register and must do a KYC.

The code of computing the ticket allocation, doing snapshots of EGLD token holders/delegators/stakers will be the same as for MEX tokens, when we computed the allocation for those.

The information about the ticket allocation per address will be written into a special launchpad contract and from there a decentralized process will start to select the winners and to resolve the token sale.

Users will have to deposit a certain amount of tokens (fixed amount per allocated ticket), known as "confirming" tickets. Users may only confirm part of their tickets if they wish to.  

Any unconfirmed tickets are filtered out, and the winner selection begins, i.e the shuffling of tickets - done using a true randomness source, directly on the blockchain.  

The random selection will use the Fisher-Yates shuffling method: https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle

The randomness seed to be used for the shuffling will be taken from the current and previous block headers random seeds, by adding those two 48 byte seeds together and hashing the result. This randomness is super secure and cannot be forged by malicious actors. 
https://medium.com/elrondnetwork/elrond-improvement-change-in-consensus-and-randomness-source-d764a3fad35

The process of the launchpad:
1) Announce the launchpad project and the days when the snapshots for eGLD delegators and stakers will happen.
2) Start the KYC process.
3) Do the snapshots and save it into a public database
4) Launch the launchpad smart contract and set-up the custom settings
5) Write the list(address, numberOfTickets) to the smart contract - only owner function.
6) Users will deposit their eGLD/bUSD (whitelisted token) and confirm part of all their allocated tickets.
7) Select the start date when the selection will happen.
8) Call "filterTickets" a few times to remove the unconfirmed tickets/tickets owned by blacklisted addresses.
9) Call “selectWinners” a few times in order to randomly shuffle all the ticket entries from the smart contract.
10) Set the number of winners and the start time for claiming the launchpad tokens
11) Users will claim the launchpad tokens.

Specifications for the smart contract:
1) Only owner function of setTickets through which the database of the SC is filled. The input is in the form of a list(address, numberOfTickets). The smart contract internally will create through storage mappers with (address - ticketIDs) and (ticketID - status). Vector of ticketIDs for winners.

2) The contract will have a set of only owner functions in order to:
- whitelist the tokenID for the final payment
- set the number of tokens to be paid per ticket
- set number of winners
- set time limit for buying tickets
- set number of tokens which are given to the user per claim ticket
- set the start date for the selection algorithm
- Reject winner - owner only - if the KY was exploited for example
- Deposit the launchpad tokens

3) The users will confirm their tickets by paying the appropriate fee, depending on the number of tickets they wish to confirm.

4) The contract will filter the unconfirmed tickets/blacklisted tickets. 

5) The contract will do the selection of the winners and the endpoint of “selectWinners” can be called by the owner after the start date. We might need to call “selectWinners” a few times, but the randomness seed used for the shuffling will be done via the current/previous random seed of the shard header from the first call.  

On the first call of selectWinners the random seed is computed from the current and previous block randomness and is saved in the storage of the smart contract
The algorithm will iterate through all the tickets and will shuffle the indexes into a new vector with all the tickets using the Fisher-Yates algorithm:

```
-- To shuffle an array a of n elements (indices 0..n-1):
for i from 0 to n−2 do
     j ← random integer such that i ≤ j < n
     exchange a[i] and a[j]
```

At every step, the contract verifies if it has enough gas to continue the next operation, if not, it will save the current index in order to start the selection process from that moment onwards on the next call
After finishing the shuffle of all the elements, the contract will have a waiting period before the winners can claim their launchpad tokens and any payment for non-winning tickets.

6) After X blocks a new endpoint is activated. claimLaunchpadTokens - this can be called only by those winners who confirmed their tickets by depositing eGLD/bUSD. This endpoint will give the actual launchpad tokens to the users and refund the losing tickets.
