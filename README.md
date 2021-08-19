# sc-launchpad-rs

## Launchpad smart contract

On the Elrond Protocol we aim to start several “launchpad” projects and for the token launch we want to create a set of smart contracts which handle in a decentralized manner the selection of token buyers. The basic idea is that the user is eligible to buy a set of lottery tickets for the given launchpad according to the number of EGLD tokens the user stakes / delegates. On top of this every user who wants to participate in the token sale event must register and must do a KYC.

The code of computing the ticket allocation, doing snapshots of EGLD token holders/delegators/stakers will be the same as for MEX tokens, when we computed the allocation for those.

The information about the ticket allocation per address will be written into a special launchpad contract and from there a decentralized process will start to select the winners and to resolve the token sale.

Once the winners are selected, using a true randomness source, directly on the blockchain, they can deposit their tokens and receive the previously set number of tokens of the launchpad project. When all the tickets have been claimed and all the tokens were deposited and paid out, the token launch event is considered finalized.

The random selection will user the Fisher-Yates shuffling method: https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle

The randomness seed to be used for the shuffling will be taken from the current and previous block headers random seeds, by adding those two 48 byte seeds together and hashing the result. This randomness is super secure and cannot be forged by malicious actors. 
https://medium.com/elrondnetwork/elrond-improvement-change-in-consensus-and-randomness-source-d764a3fad35

The process of the launchpad:
1) Announce the launchpad project and the days when the snapshots for eGLD delegators and stakers will happen.
2) Start the KYC process.
3) Do the snapshots and save it into a public database
4) Launch the launchpad smart contract and set-up the custom settings
5) Write the list(address, numberOfTickets) to the smart contract - only owner function.
6) Select the start date when the selection will happen.
7) Call “selectWinners” a few times in order to randomly shuffle all the ticket entries from the smart contract.
8) Set the number of winners and the deadline to buy the launchpad tokens
9) Users will deposit their eGLD/bUSD (whitelisted token) and buy the launchpad tokens.
10) If not all the tickets were claimed repeat number 8 and 9

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

3) The contract will do the selection of the winners and the endpoint of “selectWinners” can be called by anyone after the start date. We might need to call “selectWinners” a few times, but the randomness seed used for the shuffling will be done via the current/previous random seed of the shard header from the first call.  

On the first call of selectWinners the random seed is computed from the current and previous block randomness and is saved in the storage of the smart contract
The algorithm will iterate through all the tickets and will shuffle the indexes into a new vector with all the tickets using the Fisher-Yates algorithm:

```
-- To shuffle an array a of n elements (indices 0..n-1):
for i from 0 to n−2 do
     j ← random integer such that i ≤ j < n
     exchange a[i] and a[j]
```

At every step, the contract verifies if it has enough gas to continue the next operation, if not, it will save the current index in order to start the selection process from that moment onwards on the next call
After finishing the shuffle of all the elements, the contracts enter a new stage, the buying/selling process.

4) Now the contract enters the stage of buying/selling. The users are required to deposit the exact amount of tokens per ticket. When they buy it, only confirm the tickets. If not the exact amount is deposited, the transaction will fail.

5) There is a defined period when the tickets can be claimed, if the user comes after, the transaction will fail. If after the defined period ends and there are still unclaimed tickets, the next batch from the random shuffle will be able to deposit and confirm the tickets and again with a defined period. This thing will go on until there are no more winning tickets to be bought.

6) After X epochs a new endpoint is activated. claimLaunchpadTokens - this can be called only by those winners who confirmed their tickets by depositing eGLD/bUSD. This endpoint will give the actual launchpad tokens to the users.
