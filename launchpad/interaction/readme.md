---
id: readme
title: Owner’s guide to Launchpad Smart Contract control
---

This document provides a description of the way the Launchpad Smart Contract behaves and how you can interact with it as its owner. 

## Launchpad Smart Contract parameters
The Launchpad Smart Contract defines a set of parameters under which it operates. These parameters are:
- **Launchpad token ID** - token awarded by the owner in exchange to the winning lottery tickets
- **Ticket payment token ID** - token ID in which the investors pay their lottery tickets
- **Ticket price** - cost to be paid by investors in Ticket Payment tokens per lottery ticket
- **Number of winning lottery tickets**
- **Number of Launchpad tokens awarded for winning ticket**
- **Blocks defining the Launchpad stages** - see next section for details


## Launchpad Smart Contract stages
The Launchpad Smart Contract operates in stages, each stage allowing/requiring certain actions to be performed by its owner account or investor accounts. The timing of each stage is configured in block numbers. This configuration is done at contract deployment, but can also be changed with some restrictions during the contract’s lifetime.

In detail, the contract is configured with 3 distinct block numbers that are defining the stages:
- “**Confirm tickets**” block
- “**Select winning tickets**” block
- “**Claim**” block

Here’s a description of each stage defined by these block numbers and what’s expected to happen in each of them:

### “Add tickets” stage
This is the first stage of the contract, happening between the moment the contract is deployed until the configured “Confirmation” block is reached. 
This stage is dedicated exclusively to give enough time to the owner to set up the initial state of the contract before the “Confirm tickets” stage starts in which investors can confirm and pay their tickets. 

Thus, expected actions on owner’s responsibility in this stage are:
- `mandatory` Add the number of eligible tickets for each potential investor account 
- `mandatory` Deposit the entirety of launchpad tokens to be awarded for the winning lottery tickets
- Adjust the ticket price if needed
	
If the mandatory actions for this stage are not performed, the contract can switch to the next stages when the time is due, but the expected actions in those stages will be blocked due to missing prerequisites.

### “Confirm tickets” stage
This is the second stage of the contract, happening between the “Confirmation” block and the “Select winning tickets” block. 
This stage is mainly the potential investors’ time to shine, giving them the chance to confirm and pay any number of lottery tickets from their eligible allocated lot.

Possible actions in this stage for the owner of the Launchpad Smart Contract are:

- Add/remove investor addresses to/from blacklist if needed - blacklisting an address basically blacklists their allocated ticket lot, so these tickets will not be participating in the winning tickets selection. Any payment already done by a blacklisted address will be refunded in the final stage of the Launchpad Smart Contract. 

As it can be observed, there are no mandatory actions to be performed on the owner's responsibility during this stage. Thus, if no blacklisting/whitelisting is necessary during this stage, you can sit back and relax.

### “Select winning tickets” stage
This is the third stage of the contract, starting after the “Select winning tickets” block is reached.

Based on the configuration of the “Select winning tickets” block and “Claim” block, it can last either until the winning tickets were successfully selected (in case the “Claim” block is equal to the “Select winners” block), or until the “Claim” block is reached (when the “Claim” block number is greater than the “Select winners” block).

In this stage, ticket confirmation, blacklisting and whitelisting actions are no longer possible as, from this point on, the entire ticket list should be filtered down to a list of only valid tickets from which the winning tickets are selected.

Therefore, expected actions in this stage are:

- `mandatory` Filter tickets - this action starts and runs a ticket filtering process which is removing the unpaid and blacklisted lottery tickets.
The entire process is a lengthy one, so it is designed to be performed as much as possible within the given gas and then pause so that it can be resumed with the next trigger. Due to this reason, this action should be repeated several times until the filtering process is successfully finished.
- `mandatory` Select winning tickets - this action starts and runs the winning tickets selection from the filtered list. It cannot start without a previously completed “Filter tickets” process and, similarly to the “filter tickets” action, it has to be repeated several times until the winning tickets selection process is successfully finished.
	
These two actions can be performed by any account, be it the owner of the Launchpad Smart Contract or not. Therefore, anyone can contribute to the advancement to the next stage.

### “Claim” stage
This is the fourth and final stage of the Launchpad Smart Contract and it is happening either after the “winning tickets selection” process is successfully finished (when the “Claim” block is equal to the “Select winning tickets” block) or after the “Claim” block is reached (in case the “Claim” block number is bigger than the “Select winning tickets” block).
During this stage, all participating accounts can claim their winning tokens and refunds for lottery tickets that didn’t make it.

As the owner of the Launchpad Smart Contract, the possible action in this stage is:

- Claim ticket payment - this action will transfer all paid tokens for the winning lottery tickets to the owner of the Launchpad Smart Contract. 
In the unlikely event that the total number of paid tickets is less than the configured number of winning tickets, the unspent launchpad tokens will be refunded to the owner of the Launchpad Smart Contract as well besides the gathered payment for the winning tickets.


## Let’s get physical
It may sound complicated in theory, but in practice you’ll be funding your business without even breaking a sweat. You can find below a practical example on how things will actually happen for real.

First, we will deploy and parameterize the Launchpad Smart Contract in advance for you, so you don’t have to worry about this part. We’ll add the eligible tickets for potential investors as well and fine tune the final ticket price, then we’ll hand the ownership of the Launchpad Smart Contract to you along with a “ready to use” script file to control the Launchpad Smart Contract easily.


### Prerequisites
Erdpy will be the base tool used to send the necessary transactions that are interacting with the Launchpad Smart Contract, so you’ll have to install it. Besides, it will be handy to have it anyway since you’re planning to work within the Elrond ecosystem, so this sets you up for the future as well.
Go ahead and follow the instructions on this link and you’ll be up and running in no time: 

[Erdpy install procedure](https://docs.elrond.com/sdk-and-tools/erdpy/installing-erdpy/)

After you’ve successfully installed erdpy, prepare an account to which we can transfer the Launchpad Smart Contract ownership and please care that you should have your launchpad tokens ready in this very same account, along with a small amount of EGLD for the transaction fees.

All that’s left to prepare is a folder in which you should place your PEM file containing the private key to your Launchpad Smart Contract owner account along with the end_owner_snippets.sh file we will build for your contract and provide to you.

```
launchpad-control-folder/
├── end_owner_snippets.sh
└── myOwnerAccount.pem
```

Now, every time you intend to interact with the Launchpad Smart Contract, open a terminal window pointing into this folder, execute 
```
source end_owner_snippets.sh 
```
and then execute your desired action. Here we go:


### Deposit the launchpad tokens
You will receive the ownership of the Launchpad Smart Contract before the “Confirm tickets” block starts. At this point, everything you’re expected to do for the things to go on is to deposit the launchpad tokens. 

To get this action done, write in the prepared terminal the following command:
```
depositLaunchpadTokens
```
And that’s it, done. You can go ahead and check the status of the submitted transaction in the Elrond explorer.

### Add address to blacklist
This one requires a bit more effort as we’re not yet able to guess the future and foresee the address you need to blacklist, so you’ll have to provide it yourself. Therefore, in the prepared terminal window, execute the following command (just replace the `address_to_blacklist` with the real address you have in mind):
```
addAddressToBlacklist address_to_blacklist
```
Done. You can go ahead and check the status of the submitted transaction in the Elrond explorer.

### Remove address from blacklist
Similarly to the way you’ve added an address to the blacklist, you can also remove it with the following command (just replace the `address_to_remove_from_blacklist` with the real address you have in mind)
```
removeAddressFromBlacklist address_to_remove_from_blacklist
```
### Filter tickets
After the contract enters the “Select winning tickets” stage, you’ll be able to start filtering the lottery tickets. In order to do this, you have to execute the following command in the prepared terminal window:
```
filterTickets
```
As described above, the ticketing filtering process is lengthy and it requires the filterTickets action to be repeated until successfully completed.
When the filtering process is not yet completed, the Transaction Smart Contract Result will show `@ok@696e746572727570746564`(“**interrupted**” in hex encoding), so you’ll know that you have to repeat the **filterTickets** command.

The process is finished when the Transaction Smart Contract Result for the executed transaction contains `@ok@636f6d706c65746564` (“**completed**” in hex encoding). You’ll also know that it’s done when any further attempted filterTickets transaction will fail with the reason “**Tickets already filtered**”. 

As a fun bonus, since this action is open to execute by anyone, you might be racing with someone else to be the first to get the “**completed**” result.

### Select winning tickets
In order to select the winning tickets, you have to execute the following command in the prepared terminal window:
```
selectWinners
```
When the selection of the winning tickets is not yet completed, the Transaction Smart Contract Result will show `@ok@696e746572727570746564` (“**interrupted**” in hex encoding), so you’ll have to repeat the selectWinners command.

You’ll know when to stop when the Transaction Smart Contract Result for the executed transaction contains `@ok@636f6d706c65746564` (“**completed**” in hex encoding) or when the transaction itself will fail with the reason “**Not in winner selection period**” or “**Winners already selected**”, which is expected to happen in case someone successfully finished it already. 

Similarly to the filter tickets action, you might be racing with someone else to the “**completed**” result since this action is open to execute by anyone.

### Claim ticket payments
The last and most important command. When the “Claim” stage is reached, just execute the following command in the prepared terminal window:
```
claimTicketPayment
```
Go ahead and check the Elrond explorer for the status of this transaction. This is the big one! 
If everything is fine, then congrats! You’ve reached your destination on the Elrond Launchpad.



Obviously, if anything goes wrong or if you have any difficulties during the run of the Launchpad campaign, we’ll be with you.
