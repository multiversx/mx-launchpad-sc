OWNER_PEM_PATH="ownerWallet.pem"

ADDRESS="erd1qqqqqqqqqqqqqpgqmn5m0q4mrfr4s3dmxxce869rrvmfcuukdg0qfvggs6"
PROXY=https://devnet-gateway.multiversx.com
CHAIN_ID="D"

LAUNCHPAD_TOKEN_ID="DLNTK-79679c"

LAUNCHPAD_TOKENS_AMOUNT_TO_DEPOSIT_HEX=0x02faf080 # Amount should be equal to NR_WINNING_TICKETS * LAUNCHPAD_TOKENS_PER_WINNING_TICKET


# "ADD TICKETS" STAGE ACTIONS BELOW

depositLaunchpadTokens() {
    local ENDPOINT_NAME_HEX="0x$(echo -n 'depositLaunchpadTokens' | xxd -p -u | tr -d '\n')"
    local LAUNCHPAD_TOKEN_ID_HEX="0x$(echo -n ${LAUNCHPAD_TOKEN_ID} | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=15000000 --function="ESDTTransfer" \
    --arguments ${LAUNCHPAD_TOKEN_ID_HEX} ${LAUNCHPAD_TOKENS_AMOUNT_TO_DEPOSIT_HEX} ${ENDPOINT_NAME_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# "CONFIRM TICKETS" STAGE ACTIONS BELOW

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

# "CLAIM" STAGE ACTIONS BELOW

claimTicketPayment() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=25000000 --function="claimTicketPayment" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}