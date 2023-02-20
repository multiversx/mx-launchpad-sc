OWNER_PEM_PATH=""

ADDRESS="erd1qqqqqqqqqqqqqpgqww6yy7u5fddnejxf0m5z7qtluu06c8g6ad7q2zddq3"
PROXY=https://devnet-gateway.multiversx.com
CHAIN_ID="D"

LAUNCHPAD_TOKEN_ID="LNCHPD-c4577b"
LAUNCHPAD_TOKENS_PER_WINNING_TICKET=5000000000000000000
TICKET_PAYMENT_TOKEN="EGLD"
TICKET_PRICE=10000000000000000 # 0.01
NR_WINNING_TICKETS=10
LAUNCHPAD_TOKENS_AMOUNT_TO_DEPOSIT_HEX=0x02b5e3af16b1880000   # Amount should be equal to NR_WINNING_TICKETS * LAUNCHPAD_TOKENS_PER_WINNING_TICKET
CONFIRMATION_PERIOD_START_BLOCK=47
WINNER_SELECTION_START_BLOCK=48
CLAIM_START_BLOCK=48
NFT_COST="0x000000000445474c44000000000000000000000007470DE4DF820000" # 0.02 EGLD
NFT_COST_DEC=20000000000000000
TOTAL_NFTS=2


build() {
    erdpy contract clean ../../launchpad
    erdpy contract build ../../launchpad
}

deploy() {
    local TICKET_PAYMENT_TOKEN_HEX="0x$(echo -n ${TICKET_PAYMENT_TOKEN} | xxd -p -u | tr -d '\n')"
    local LAUNCHPAD_TOKEN_ID_HEX="0x$(echo -n ${LAUNCHPAD_TOKEN_ID} | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract deploy --bytecode="../output/launchpad-with-nft.wasm" --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=200000000 \
    --arguments ${LAUNCHPAD_TOKEN_ID_HEX} ${LAUNCHPAD_TOKENS_PER_WINNING_TICKET} \
    ${TICKET_PAYMENT_TOKEN_HEX} ${TICKET_PRICE} ${NR_WINNING_TICKETS} \
    ${CONFIRMATION_PERIOD_START_BLOCK} ${WINNER_SELECTION_START_BLOCK} ${CLAIM_START_BLOCK} \
    ${NFT_COST} ${TOTAL_NFTS} \
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

# params
#   $1 = User pem
#   $2 = Amount in hex
addTicketsPEM() {
    local USER_ADDRESS_HEX="0x$(erdpy wallet pem-address-hex $1)"

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
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="setTicketPrice" \
    --arguments $1 \
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

# params
#   $1 = Token name
#   $2 = Token ticker
issueMysterySft() {
    TOKEN_NAME_HEX="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"
    TOKEN_TICKER_HEX="0x$(echo -n $2 | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=100000000 --function="issueMysterySft" --value=50000000000000000\
    --arguments $TOKEN_NAME_HEX $TOKEN_TICKER_HEX \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
} 

# params
createInitialSfts() {
    TOKEN_NAME_HEX="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"
    TOKEN_TICKER_HEX="0x$(echo -n $2 | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="createInitialSfts" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
} 

# params
setInitialTransferRole() {

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=100000000 --function="setTransferRole" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
} 

# params
#   $1 = Address
setTransferRole() {
    local ADDRESS_HEX="0x$(erdpy wallet bech32 --decode $1)"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=100000000 --function="setTransferRole" \
    --arguments ${ADDRESS_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
} 


# "CONFIRM TICKETS" STAGE ENDPOINTS BELOW

# params
#   $1 = User address
addAddressToBlacklist() {
    local USER_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode $1)"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=15000000 --function="addAddressToBlacklist" \
    --arguments ${USER_ADDRESS_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# params
#   $1 = User address
removeAddressFromBlacklist() {
    local USER_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode $1)"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=15000000 --function="removeAddressFromBlacklist" \
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

selectNftWinners() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=550000000 --function="selectNftWinners" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# "CLAIM" STAGE ENDPOINTS BELOW

claimTicketPayment() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=25000000 --function="claimTicketPayment" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

claimNftPayment() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=25000000 --function="claimNftPayment" \
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
confirmNftUser() {
    local PADDING="0x"
    local PAYMENT_AMOUNT=$NFT_COST_DEC

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=$1 --pem-index=$2\
    --gas-limit=20000000 --function="confirmNft" --value=${PAYMENT_AMOUNT} \
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

upgrade() {
    erdpy --verbose contract upgrade ${ADDRESS} \
    --bytecode="../output/launchpad_upgrade.wasm" --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=50000000 \
    --send --outfile="upgrade-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade_old() {
    local TICKET_PAYMENT_TOKEN_HEX="0x$(echo -n ${TICKET_PAYMENT_TOKEN} | xxd -p -u | tr -d '\n')"
    local LAUNCHPAD_TOKEN_ID_HEX="0x$(echo -n ${LAUNCHPAD_TOKEN_ID} | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract upgrade ${ADDRESS} \
    --bytecode="../output/launchpad-with-nft.wasm" --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=200000000 \
    --arguments ${LAUNCHPAD_TOKEN_ID_HEX} ${LAUNCHPAD_TOKENS_PER_WINNING_TICKET} \
    ${TICKET_PAYMENT_TOKEN_HEX} ${TICKET_PRICE} ${NR_WINNING_TICKETS} \
    ${CONFIRMATION_PERIOD_START_BLOCK} ${WINNER_SELECTION_START_BLOCK} ${CLAIM_START_BLOCK} \
    ${NFT_COST} ${TOTAL_NFTS} \
    --send --outfile="upgrade-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade_blacklist() {
    erdpy --verbose contract upgrade ${ADDRESS} \
    --bytecode="../output/launchpad_blacklist.wasm" --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=50000000 \
    --send --outfile="upgrade-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

# CHANGE SC OWNERSHIP

#params
#   $1 = New owner address
changeSCOwner() {
    local NEW_OWNER_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode $1)"
    
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=25000000 --function="ChangeOwnerAddress" \
    --arguments ${NEW_OWNER_ADDRESS_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}
