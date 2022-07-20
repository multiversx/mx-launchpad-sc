OWNER_PEM_PATH="elrondOwner.pem"

ADDRESS=$(erdpy data load --key=address-sc)
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID="D"

LAUNCHPAD_TOKEN_ID="DLNTK-79679c"
LAUNCHPAD_TOKENS_PER_WINNING_TICKET=5000
TICKET_PAYMENT_TOKEN="EGLD"
TICKET_PRICE=100000000000000000 # 0.1 EGLD
NR_WINNING_TICKETS=10000
LAUNCHPAD_TOKENS_AMOUNT_TO_DEPOSIT_HEX=0x02faf080   # Amount should be equal to NR_WINNING_TICKETS * LAUNCHPAD_TOKENS_PER_WINNING_TICKET
CONFIRMATION_PERIOD_START_BLOCK=1895
WINNER_SELECTION_START_BLOCK=1896
CLAIM_START_BLOCK=1896


build() {
    erdpy contract clean ../../launchpad
    erdpy contract build ../../launchpad
}

deploy() {
    local TICKET_PAYMENT_TOKEN_HEX="0x$(echo -n ${TICKET_PAYMENT_TOKEN} | xxd -p -u | tr -d '\n')"
    local LAUNCHPAD_TOKEN_ID_HEX="0x$(echo -n ${LAUNCHPAD_TOKEN_ID} | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract deploy --bytecode="../output/launchpad.wasm" --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=200000000 \
    --arguments ${LAUNCHPAD_TOKEN_ID_HEX} ${LAUNCHPAD_TOKENS_PER_WINNING_TICKET} \
    ${TICKET_PAYMENT_TOKEN_HEX} ${TICKET_PRICE} ${NR_WINNING_TICKETS} \
    ${CONFIRMATION_PERIOD_START_BLOCK} ${WINNER_SELECTION_START_BLOCK} ${CLAIM_START_BLOCK} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-sc --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

# "ADD TICKETS" STAGE ENDPOINTS BELOW

# params
#   $1 = User address
#   $2 = Amount in hex
addTickets() {
    local USER_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode $1)"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="addTickets" \
    --arguments ${USER_ADDRESS_HEX} $2 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

depositLaunchpadTokens() {
    local ENDPOINT_NAME_HEX="0x$(echo -n 'depositLaunchpadTokens' | xxd -p -u | tr -d '\n')"
    local LAUNCHPAD_TOKEN_ID_HEX="0x$(echo -n ${LAUNCHPAD_TOKEN_ID} | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=15000000 --function="ESDTTransfer" \
    --arguments ${LAUNCHPAD_TOKEN_ID_HEX} ${LAUNCHPAD_TOKENS_AMOUNT_TO_DEPOSIT_HEX} ${ENDPOINT_NAME_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# params
#   $1 = New price in hex
setTicketPrice() {
    local TICKET_PAYMENT_TOKEN_HEX="0x$(echo -n ${TICKET_PAYMENT_TOKEN} | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="setTicketPrice" \
    --arguments ${TICKET_PAYMENT_TOKEN_HEX} $1 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# params
#   $1 = New ticket payment token id
setTicketPaymentToken() {
    local PAYMENT_TOKEN_ID_HEX="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="setTicketPaymentToken" \
    --arguments PAYMENT_TOKEN_ID_HEX \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# params
#   $1 = New number of tokens per winning ticket in hex
setLaunchpadTokensPerWinningTicket() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="setLaunchpadTokensPerWinningTicket" \
    --arguments $1 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# params
#   $1 = New confirm block in hex
setConfirmationPeriodStartBlock() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="setConfirmationPeriodStartBlock" \
    --arguments $1 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
} 

# params
#   $1 = New winner selection block in hex
setWinnerSelectionStartBlock() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="setWinnerSelectionStartBlock" \
    --arguments $1 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
} 

# params
#   $1 = New claim block in hex
setClaimStartBlock() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="setClaimStartBlock" \
    --arguments $1 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
} 

# "CONFIRM TICKETS" STAGE ENDPOINTS BELOW

# params
#   $1 = User address
addUsersToBlacklist() {
    local USER_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode $1)"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=15000000 --function="addUsersToBlacklist" \
    --arguments ${USER_ADDRESS_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# params
#   $1 = User address
removeUsersFromBlacklist() {
    local USER_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode $1)"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=15000000 --function="removeUsersFromBlacklist" \
    --arguments ${USER_ADDRESS_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# "SELECT WINNING TICKETS" STAGE ACTIONS BELOW

filterTickets() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=550000000 --function="filterTickets" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

selectWinners() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=550000000 --function="selectWinners" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# "CLAIM" STAGE ENDPOINTS BELOW

claimTicketPayment() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=25000000 --function="claimTicketPayment" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# USER ENDPOINTS

# params
#   $1 = User pem file path
#   $2 = User pem index
#   $3 = Number of tickets (max. 255)
confirmTicketsUser() {
    local PADDING="0x"
    local NR_TICKETS_TO_CONFIRM=$echo"0x"$(printf "%02X" $3)
    local PAYMENT_AMOUNT=$(($TICKET_PRICE * $3))

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=$1 --pem-index=$2\
    --gas-limit=20000000 --function="confirmTickets" --value=${PAYMENT_AMOUNT} \
    --arguments ${NR_TICKETS_TO_CONFIRM} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# params
#   $1 = User pem file path
#   $2 = User pem index
claimLaunchpadTokensUser() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=$1\
    --pem-index=$2 --gas-limit=25000000 --function="claimLaunchpadTokens" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}


# ADMIN

#params
#   $1 = Support address
setSupportAddress() {
    local NEW_SUPPORT_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode $1)"
    
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=25000000 --function="setSupportAddress" \
    --arguments ${NEW_SUPPORT_ADDRESS_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

#params
#   $1 = New owner address
changeSCOwner() {
    local NEW_OWNER_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode $1)"
    
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=25000000 --function="ChangeOwnerAddress" \
    --arguments ${NEW_OWNER_ADDRESS_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}