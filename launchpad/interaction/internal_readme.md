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

- Send “Owner’s Launchpad Smart Contract control guide” to the counterpart to prepare their prerequisites
Ask the counterpart to prepare the tokens in an account onto which we’ll transfer the contract ownership at some point. They should have a PEM file for the account. 
We’ll need the **Token ID** along with the **filename** of the wallet PEM file.

### Steps

**Contract deploy & scripts config for control**
1. Get a wallet with pem file & mnemonic available, some EGLD for fees/gas

2. Go to `sc-launchpad-rs/launchpad/interaction` and copy the wallet .pem file in here

3. Edit `snippets.sh` with all launchpad contract details for deployment - this will be the “control panel” on Elrond’s side

4. Terminal into the folder

5. Execute: 
```
$ source snippets.sh
```

6. Build the contract

7. Execute: 
```
$ deploy
```

8. Check contract deploy & sanity

9. Open `end_owner_snippets.sh`

10. Add the new SC address into the `ADDRESS` field

11. Add counterpart’s wallet filename into `OWNER_PEM_PATH`

12. Fill the other Launchpad parameters

13. When ready, send `end_owner_snippets.sh` to the counterpart

14. **Link contract address to frontend**


**Add tickets in contract**
15. Go to `temp-mex-indexing`

16. Execute: 
```
$ npm install
```

17. Edit `temp-mex-indexing/launchpad/vars.js` with our wallet mnemonics, snapshot file, KYC exports, proxy, launchpad contract

18. To generate `tickets-allocation.json` execute: 
```
$ node ./launchpad/computeTickets.js
```

19. To send the `addTickets` transactions to the contract execute: 
```
$ node ./launchpad/indexTickets.js
```

**Final adjustments & ownership change**
20. Adjust the final ticket price if required by executing from the snippets: 
```
$ setTicketPrice new_price_in_hex
```

21. Change ownership of the contract by executing: 
```
$ changeSCOwner counterpart_address
```

**Counterpart actions**
22. Ask the counterpart to deposit tokens after ownership transfer. For this, they should execute:
```
$ depositLaunchpadTokens
```

*At this point, insert beer and have some sleep, you deserve it (or more likely need it)*

23. After ticket confirm epoch is reached, if blacklisting is needed, counterpart should execute it via:
```
$ addAddressToBlacklist user_address
```

**Winners selection**
24. Call the following functions several times each until completed:
```
$ filterTickets
$ selectWinners
```

**Claim**
25. Ask the counterpart to claim by executing:
```
$ claimTicketPayment
```