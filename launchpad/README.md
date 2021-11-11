# Technical documentation

This is the technical documentation for the smart contract. For a more general description, please refer to the root readme file.  

# Setup

A good part of the setup is done at the deploy time, through the init function:
```
#[init]
fn init(
    &self,
    launchpad_token_id: TokenIdentifier,
    launchpad_tokens_per_winning_ticket: BigUint,
    ticket_payment_token: TokenIdentifier,
    ticket_price: BigUint,
    nr_winning_tickets: usize,
    winner_selection_start_epoch: u64,
    confirmation_period_start_epoch: u64,
    confirmation_period_in_epochs: u64,
    claim_start_epoch: u64,
) -> SCResult<()>
```

`launchpad_token_id` is the identifier of the token you want to launch through this smart contract. Keep in mind that the current implementation supports fungible ESDTs only.  

`launchpad_tokens_per_winning_ticket` is the amount of tokens that each user can claim for each of their winning ticket.  

`ticket_payment_token` is the token in which you want tickets to be paid. This will usually be "EGLD", but any other fungible ESDT can be used.  

`ticket_price` is the amount of the previously defined token the user has to pay to confirm a ticket.  

`nr_winning_tickets` is the total number of winning tickets.  

`winner_selection_start_epoch` is the start of the winning selection period, which also signals the end of the adding tickets stage.  

`confirmation_period_start_epoch` is the start of the ticket confirmation period. In this period, users "confirm" their tickets by paying the specific amount defined above.

`confirmation_period_in_epochs` is how long each confirmation period lasts.  

`claim_start_epoch` is the epoch at which the claim endpoint activates.  

Almost all of the above parameters can be changed by the owner through their specific functions at any point.  

***

Besides the initial setup, which is done at deployment time, some additional post-deploy setup is needed to be done by the owner. Before the `winner_selection_period` starts, the owner can add tickets for users through the following endpoint:
```
#[only_owner]
#[endpoint(addTickets)]
fn add_tickets(
    &self,
    #[var_args] address_number_pairs: VarArgs<MultiArg2<Address, usize>>,
) -> SCResult<()>
```

This endpint accepts pairs of `(user_address, number_of_tickets)`. The only restriction is tickets cannot be added twice for the same user, even by different calls.  

This restriction is in place because the SC stores the tickets for each user as a range `(first_ticket_id, last_ticket_id)` instead of storing each ticket ID under a different entry with its owner's address. This greatly optimizes the process of adding and retrieving the tickets for a certain user, but comes with the restriction stated before.  

The only thing that's left is to deposit the actual tokens, which is done through the following endpoint:
```
#[only_owner]
#[payable("*")]
#[endpoint(depositLaunchpadTokens)]
fn deposit_launchpad_tokens(
    &self,
) -> SCResult<()>
```

No additional arguments. Keep in mind you have to pay exactly `nr_winning_tickets * launchpad_tokens_per_winning_ticket`, otherwise, the SC will throw an error.  

# General workflow

The general workflow looks like this:
1) Add Tickets
2) Winner Selection
3) Confirm Tickets
4) Repeat steps 2 and 3 until all tickets are confirmed
5) Claim Tokens

The Add Tickets stage was described above, so we will start with the Winner Selection stage.  

## Winner Selection

Once the `AddTickets` period has concluded and the `WinnerSelection` stage has started, anyone may call the `selectWinners` endpoint to shuffle the tickets:

```
[endpoint(selectWinners)]
fn select_winners(&self) -> SCResult<OperationCompletionStatus>
```

This endpoint requires no arguments. Keep in mind this is a very expensive operation in terms of gas, so this endpoint might have to be called many times before the shuffling is complete. The shuffling is done through the [Fisher-Yates shuffle](https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle).  

`OperationCompletionStatus` is `completed` if the operation was fully completed or `interrupted` if the SC had to save progress and resume in another call.  

## Confirm Tickets

Once the winners have been selected, they may call the `confirmTickets` endpoint:
```
#[payable("*")]
#[endpoint(confirmTickets)]
fn confirm_tickets(
    &self,
    nr_tickets_to_confirm: usize,
) -> SCResult<()>
```

A confirmed ticket remains confirmed forever (unless the user is added to the blacklist, which we'll discuss more about later). The user must pay exactly `ticket_price * nr_tickets_to_confirm` of `ticket_payment_token`s. Keep in mind users are not required to confirm all tickets all at once, or even confirm them all. Any unconfirmed tickets get redistributed in the next selection period.  

## Select New Winners & Restart Confirmation Period

If the confirmation period has ended and there are still tickets left, anyone may call the `selectNewWinners` endpoint to redistribute the leftover tickets:
```
#[endpoint(selectNewWinners)]
fn select_new_winners(&self) -> SCResult<OperationCompletionStatus>
```

This operation is a lot less expensive than the initial `selectWinners`, as all this does is move the "range" of winning tickets (all tickets were initially shuffled in `selectWinners`) and mark those as winning for the current generation.  

A "generation" refers to confirm periods. A winning ticket from the first generation is not eligible for confirming from the second generation and onward.  

Once `selectNewWinners` is complete, the SC will automatically mark the start of the next Confirm stage.  

## Claim

Claim period can be reached either "naturally" by having all tickets confirm, or by being forced by the owner through the `forceClaimPeriodStart` endpoint:
```
#[only_owner]
#[endpoint(forceClaimPeriodStart)]
fn force_claim_period_start(&self) -> SCResult<()>
```

All this endpoint does is set the number of "Confirmed" tickets to be equal to the total number of winning tickets. Keep in mind this does not affect the `claim_start_epoch` parameter in any way and the endpoint might not be activate right away if that epoch has not passed yet.  

Once the claim period has started, users may claim their launchpad tokens by calling the following endpoint:
```
#[endpoint(claimLaunchpadTokens)]
fn claim_launchpad_tokens(&self) -> SCResult<()>
```

No additional arguments or payment required. The user will receive `launchpad_tokens_per_winning_ticket * nr_confirmed_tickets` launchpad tokens. 

## Special Cases

Since this whole flow requires the user to do an off-chain KYC (Know Your Customer), the SC provides blacklist functionality for cases where the user provided false information in the KYC process or other things of that nature. The owner may add users to the blacklist through the following endpoint:
```
#[only_owner]
#[endpoint(addAddressToBlacklist)]
fn add_address_to_blacklist(&self, address: Address) -> SCResult<()>
```

This disables the confirm and claim functionalities for the specifed user. If the user already paid and confirmed tickets, the owner can refund the payment through:
```
#[only_owner]
#[endpoint(refundConfirmedTickets)]
fn refund_confirmed_tickets(&self, address: Address) -> SCResult<()>
```

For cases where there has simply been a mistake or anything of that nature, the owner can remove the user from the blacklist:
```
#[only_owner]
#[endpoint(removeAddressFromBlacklist)]
fn remove_address_from_blacklist(&self, address: Address) -> SCResult<()>
```

# Conclusion

This SC can be used to perform a token launch in a decentralized manner. Of course, the initial setup is still very much centralized, but users don't have to pay upfront for tickets and there is no risk of "rigged" winners, as the SC uses a secure randomness source.  
