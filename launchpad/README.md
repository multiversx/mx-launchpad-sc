# Technical documentation

This is the technical documentation for the smart contract. For a more general description, please refer to the root readme file.  

# Setup

A good part of the setup is done at the deploy time, through the init function:
```rust
#[init]
fn init(
    &self,
    launchpad_token_id: TokenIdentifier,
    launchpad_tokens_per_winning_ticket: BigUint,
    ticket_payment_token: EgldOrEsdtTokenIdentifier,
    ticket_price: BigUint,
    nr_winning_tickets: usize,
    confirmation_period_start_block: u64,
    winner_selection_start_block: u64,
    claim_start_block: u64,
)
```

`launchpad_token_id` is the identifier of the token you want to launch through this smart contract. Keep in mind that the current implementation supports fungible ESDTs only.  

`launchpad_tokens_per_winning_ticket` is the amount of tokens that each user can claim for each of their winning ticket.  

`ticket_payment_token` is the token in which you want tickets to be paid. This will usually be "EGLD", but any other fungible ESDT can be used.  

`ticket_price` is the amount of the previously defined token the user has to pay to confirm a ticket.  

`nr_winning_tickets` is the total number of winning tickets.  

`confirmation_period_start_block` is the start of the ticket confirmation period. In this period, users "confirm" their tickets by paying the specific amount defined above and are eligible for participating in the selection with that many tickets.

`winner_selection_start_block` is the start of the winning selection period, which consists of filter tickets + select winners.

`claim_start_block` is the block at which the claim endpoint activates.  

Almost all of the above parameters can be changed by the owner through their specific functions at any point.  

***

Besides the initial setup, which is done at deployment time, some additional post-deploy setup is needed to be done by the owner. Before the `confirmation_period` starts, the owner can add tickets for users through the following endpoint:
```rust
#[only_owner]
#[endpoint(addTickets)]
fn add_tickets(
    &self,
    address_number_pairs: MultiValueEncoded<MultiValue2<Address, usize>>,
) 
```

This endpint accepts pairs of `(user_address, number_of_tickets)`. The only restriction is tickets cannot be added twice for the same user, even by different calls.  

This restriction is in place because the SC stores the tickets for each user as a range `(first_ticket_id, last_ticket_id)` instead of storing each ticket ID under a different entry with its owner's address. This greatly optimizes the process of adding and retrieving the tickets for a certain user, but comes with the restriction stated before.  

The only thing that's left is to deposit the actual tokens, which is done through the following endpoint:
```rust
#[only_owner]
#[payable("*")]
#[endpoint(depositLaunchpadTokens)]
fn deposit_launchpad_tokens(
    &self,
) 
```

No additional arguments. Keep in mind you have to pay exactly `nr_winning_tickets * launchpad_tokens_per_winning_ticket`, otherwise, the SC will throw an error.  

# General workflow

The general workflow looks like this:
1) Add Tickets
2) Confirm Tickets
3) Filter Tickets - repeat until completed
4) Winner Selection - repeat until completed
5) Claim Tokens

The Add Tickets stage was described above, so we will start with the Winner Selection stage.

## Confirm Tickets

Once all tickets have been adding and the confirm tickets period started, users ca confirm their tickets through the `confirmTickets` endpoint.
```rust
#[payable("*")]
#[endpoint(confirmTickets)]
fn confirm_tickets(
    &self,
    nr_tickets_to_confirm: usize,
) 
```

A confirmed ticket remains confirmed forever (unless the user is added to the blacklist, which we'll discuss more about later). The user must pay exactly `ticket_price * nr_tickets_to_confirm` of `ticket_payment_token`s. Keep in mind users are not required to confirm all tickets all at once, or even confirm them all. Any unconfirmed tickets get filtered before the winner selection.

## Filter Tickets

Before the winner selection can start, unconfirmed tickets have to be filtered. This step is necessary to not over-complicate the winner selection logic. This endpoint can be called by anyone, and it must be called multiple times until all tickets were filtered.

```rust
#[endpoint(filterTickets)]
fn filter_tickets(&self) -> OperationCompletionStatus
```

## Winner Selection

After tickets were filtered, we only have the confirmed tickets, which are shuffled through the `selectWinners` endpoint.

```rust
[endpoint(selectWinners)]
fn select_winners(&self) -> OperationCompletionStatus
```

This endpoint requires no arguments. Keep in mind this is a very expensive operation in terms of gas, so this endpoint might have to be called many times before the shuffling is complete. The shuffling is done through the [Fisher-Yates shuffle](https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle).  

`OperationCompletionStatus` is `completed` if the operation was fully completed or `interrupted` if the SC had to save progress and resume in another call.  

This endpoint can also be called by anyone.

## Claim

Once the claim period has started, users may claim their launchpad tokens by calling the following endpoint:
```rust
#[endpoint(claimLaunchpadTokens)]
fn claim_launchpad_tokens(&self) 
```

No additional arguments or payment required. The user will receive `launchpad_tokens_per_winning_ticket * nr_winning_tickets` launchpad tokens, and also have their non-winning tickets refunded.

## Special Cases

Since this whole flow requires the user to do an off-chain KYC (Know Your Customer), the SC provides blacklist functionality for cases where the user provided false information in the KYC process or other things of that nature. The owner may add users to the blacklist through the following endpoint:
```rust
#[only_owner]
#[endpoint(addUsersToBlacklist)]
fn add_users_to_blacklist(&self, users_list: MultiValueEncoded<ManagedAddress>)
```

The confirmed tickets are automatically refunded.

For cases where there has simply been a mistake or anything of that nature, the owner can remove the users from the blacklist:
```
#[only_owner]
#[endpoint(removeUsersFromBlacklist)]
fn remove_users_from_blacklist(&self, users_list: MultiValueEncoded<ManagedAddress>)
```

# Conclusion

This SC can be used to perform a token launch in a decentralized manner. Of course, the initial setup is still very much centralized, but there is no risk of "rigged" winners, as the SC uses a secure randomness source.  
