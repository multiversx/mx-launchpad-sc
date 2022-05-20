---
id: readme
title: Internal launchpad SC setup & deploy procedure
---

This document provides a description of the Launchpad Smart Contract setup, deploy, integration & flow procedures

## Internal launchpad SC setup & deploy procedure

### Prerequisites

- Clone the launchpad smart contract repo:
https://github.com/ElrondNetwork/sc-launchpad-rs

- Clone temp-mex-indexing repo:
https://github.com/ElrondNetwork/temp-mex-indexing

- We have an “Owner’s Launchpad Smart Contract control guide” for the startups to prepare their prerequisites.
Ask the counterpart to prepare the tokens in an account onto which we’ll transfer the contract ownership at some point before entering into the "Confirmation" epoch. They should either have a PEM file prepared for the account if they plan to use the snippets, or issue the necessary transactions from the wallet directly (it's easy).

- We’ll need the **Token ID** to setup the contract and, in case they plan to use the snippet, the **filename** of the wallet PEM file.

### Steps

#### Contract deploy & scripts config for control

1. Get a wallet with pem file & mnemonic available, some EGLD for fees/gas

2. Go to `sc-launchpad-rs/launchpad/interaction` and copy the wallet .pem file in here

3. Edit `snippets.sh` with all launchpad contract details for deployment - this will be the “control panel” on Elrond’s side

4. Terminal into the folder

5. Execute:
```
$ source snippets.sh
```

6. Build the contract if you didn't do it already

7. Execute:
```
$ deploy
```

8. Check contract deployment tx & contract sanity

9. **Link contract address to frontend & whitelist the contract address for payments**

**TODOs in case counterpart owner plans to use the contract snippets"**

10. Open `end_owner_snippets.sh`

11. Add the new SC address into the `ADDRESS` field

12. Add counterpart’s wallet filename into `OWNER_PEM_PATH`

13. Fill the other Launchpad parameters

14. When ready, send `end_owner_snippets.sh` to the counterpart


#### Add tickets in contract

15. First, we need to filter the KYC list from the snapshots, then import the resulting list into the contract. Go to `temp-mex-indexing`

16. Execute:
```
$ npm install
```

17. Edit `temp-mex-indexing/launchpad/vars.js` with our wallet mnemonics, snapshot file, KYC exports, proxy, launchpad contract

  **KYC status has to be exported from our DataBase**, most likely, to include changed addresses after KYC submit. The data needed here is: Address, Country of residence (for capped ticket owners).

  Theoretically, the data in our DataBase should be combined with the data from the KYC processor using the "email" as primary key.

18. To generate the accounts with according tickets allocation `tickets-allocation.json` execute:
```
$ node ./launchpad/computeTickets.js
```

19. To send the `addTickets` transactions to the contract execute according to the resulted tickets allocation:

  It is recommended to delete any unused/old files in the folder to avoid confusions / wrong files used by the import.
```
$ node ./launchpad/indexTickets.js
```
  **Care for the proper path to the filtered tickets file as this path is currently hardcoded in indexTickets.js in function getTicketData.**

  In case the import fails (e.g. unresponsive API), check the last processed transaction by the contract, get the last added account from this tx then delete all the addresses above this one (including it) from the filtered accounts list that is used for the import. Then restart the import.

#### Final adjustments & ownership change

20. Adjust the final ticket price if required by executing from the snippets:
```
$ setTicketPrice new_price_in_hex
```
 - **Set final ticket price in web interface config - careful not to cause the ticket dividing issue**


21. Change ownership of the contract by executing:
```
$ changeSCOwner counterpart_address
```

#### Counterpart actions

22. Ask the counterpart to deposit tokens after ownership transfer. For this, they should execute from snippet:
```
$ depositLaunchpadTokens
```
Or via normal tx towards the contract address with Gas Limit: `7000000` and data:
```
ESDTTransfer@[token_id_in_hex]@[token_amount_in_hex]@6465706F7369744C61756E6368706164546F6B656E73
```

*Time to have a good sleep.*

23. After ticket confirm epoch is reached, if blacklisting is needed, counterpart should execute it by snippet via:
```
$ addAddressToBlacklist user_address
```
Or via normal tx towards the contract address with Gas Limit: `7000000` and data:
```
addAddressToBlacklist@[user_address_in_hex]
```

#### Winners selection

24. Call the following functions several times each until completed:
```
$ filterTickets
```
```
$ selectWinners
```
Or via normal tx towards the contract address with Gas Limit: `7000000` and data:
```
filterTickets
```
```
selectWinners
```

#### Claim

25. Ask the counterpart to claim by executing:
```
$ claimTicketPayment
```
Or via normal tx towards the contract address with Gas Limit: `7000000` and data:
```
claimTicketPayment
```

All done.