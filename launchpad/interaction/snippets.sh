OWNER_PEM_PATH=""
USER1_PEM_PATH=""
USER2_PEM_PATH=""

USER1_HEX_ADDRESS=0x
USER2_HEX_ADDRESS=0x

ADDRESS=$(erdpy data load --key=address-sc)
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID=D

LAUNCHPAD_TOKEN_ID=0x
LAUNCHPAD_TOKENS_PER_WINNING_TICKET=100
TICKET_PAYMENT_TOKEN=0x45474c44 # "EGLD"
TICKET_PRICE=1000000000000000000 # 1 EGLD
NR_WINNING_TICKETS=5
CONFIRMATION_PERIOD_START_EPOCH=1000
WINNER_SELECTION_START_EPOCH=2000
CLAIM_START_EPOCH=3000

build() {
    erdpy contract clean ../launchpad
    erdpy contract build ../launchpad
}

deploy() {
    erdpy --verbose contract deploy --bytecode="output/launchpad.wasm" --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 \
    --arguments ${LAUNCHPAD_TOKEN_ID} ${LAUNCHPAD_TOKENS_PER_WINNING_TICKET} \
    ${TICKET_PAYMENT_TOKEN} ${TICKET_PRICE} ${NR_WINNING_TICKETS} \
    ${CONFIRMATION_PERIOD_START_EPOCH} ${WINNER_SELECTION_START_EPOCH} ${CLAIM_START_EPOCH} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-sc --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

depositLaunchpadTokens() {
    local AMOUNT=0x # Amount should be equal to NR_WINNING_TICKETS * LAUNCHPAD_TOKENS_PER_WINNING_TICKET
    local ENDPOINT_NAME=0x6465706f7369744c61756e6368706164546f6b656e73 # depositLaunchpadTokens - do not change

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=15000000 --function="ESDTTransfer" \
    --arguments ${LAUNCHPAD_TOKEN_ID} ${AMOUNT} ${ENDPOINT_NAME} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addTickets() {
    local NR_TICKETS1=6
    local NR_TICKETS2=4

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=20000000 --function="addTickets" \
    --arguments ${USER1_HEX_ADDRESS} ${NR_TICKETS1} ${USER2_HEX_ADDRESS} ${NR_TICKETS2} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

confirmTicketsUser1() {
    local NR_TICKETS_TO_CONFIRM=1
    local PAYMENT_AMOUNT=$(($TICKET_PRICE * $NR_TICKETS_TO_CONFIRM))

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${USER1_PEM_PATH} \
    --gas-limit=15000000 --function="confirmTickets" --value=${PAYMENT_AMOUNT} \
    --arguments ${NR_TICKETS_TO_CONFIRM} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

confirmTicketsUser2() {
    local NR_TICKETS_TO_CONFIRM=1
    local PAYMENT_AMOUNT=$(($TICKET_PRICE * $NR_TICKETS_TO_CONFIRM))

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${USER2_PEM_PATH} \
    --gas-limit=15000000 --function="confirmTickets" --value=${PAYMENT_AMOUNT} \
    --arguments ${NR_TICKETS_TO_CONFIRM} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

filterTickets() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=25000000 --function="filterTickets" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

selectWinners() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${OWNER_PEM_PATH} \
    --gas-limit=25000000 --function="selectWinners" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

claimLaunchpadTokensUser1() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${USER1_PEM_PATH} \
    --gas-limit=25000000 --function="claimLaunchpadTokens" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

claimLaunchpadTokensUser2() {
    # no arguments needed
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${USER2_PEM_PATH} \
    --gas-limit=25000000 --function="claimLaunchpadTokens" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}
