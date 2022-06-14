---
id: readme
title: Internal launchpad SC setup & deploy procedure
---

This document provides a description of the Launchpad V2 Smart Contract setup, deploy, integration & flow procedures.
It contains only a superseding specification over the internal_readme.md of the original Launchpad SC. Thus, the steps described in here should be applied over the ones specified by the original document.

## Internal launchpad SC setup & deploy procedure

### Prerequisites

- In addition to the original prerequisites, we’ll need the **Token Name and Ticker** for the SFT the contract will create to manage the Mistery Box sell.

### Steps

#### Contract deploy & scripts config for control

1. Get a wallet with pem file & mnemonic available, some EGLD for fees/gas

2. Go to `sc-launchpad-rs/launchpad-with-nft/interaction` and copy the wallet .pem file in here

3. Edit `snippets.sh` with all launchpad contract details for deployment - this will be the “control panel” on Elrond’s side. Care for the NFT_COST parameter, as it is a merged formats input.

8. After deployment, setup the Mistery box handling then proceed with contract checking for sanity:

 - execute the following commands and care to wait for complete processing in between them:
 ```
$ issueMysterySft [SFT-Name] [SFT-Ticket]
$ createInitialSfts
$ setInitialTransferRole
 ```

#### Winners selection

24. In additionl to the original functions, call function until completed:
```
$ selectNftWinners
```
Or via normal tx towards the contract address with Gas Limit: `7000000` and data:
```
selectNftWinners
```

#### Claim

25. In addition to the original claim, ask the counterpart to claim the mistery box payment by executing:
```
$ claimNftPayment
```
Or via normal tx towards the contract address with Gas Limit: `7000000` and data:
```
claimNftPayment
```

#### Further actions related to handling of the SFT (e.g. additional mistery box contract)

26. In order to whitelist transfers of the SFT towards a specified address that can manage the mistery box functionality (e.g. IGO Smart Contract), execute the following function from the Launchpad V2 SC:
```
$ setTransferRole [New-SFT-Handling-Address]
```


All done.
