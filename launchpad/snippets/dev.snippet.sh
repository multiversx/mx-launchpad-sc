PEM="$HOME/pems/dev.pem"
ROOT="$HOME/Elrond/sc-launchpad-rs"
SCRIPT_PATH="$HOME/Elrond/sc-launchpad-sc/launchpad/snippets"

ADDRESS=$(erdpy data load --key=address-devnet)
DEV_PROXY=https://devnet-gateway.elrond.com

GAS=150_000_000

MPAD_TICK=0x4d5041442d633636343436
AMOUNT=1000000000000000000
DEP_ENDPOINT=0x6465706f7369744c61756e6368706164546f6b656e73

USER_ADDR=0x75cb87c24351a67b892f57dcec0eb2b2a07aafab2f1aab741a10fc61059f2fe8

build_sc() {
  cd ${ROOT}

  erdpy contract clean ./launchpad
  erdpy contract build ./launchpad

  cd ${SCRIPT_PATH}
}

deploy() {
  erdpy contract deploy --bytecode=${ROOT}/launchpad/output/launchpad.wasm --pem=${PEM} --proxy=${DEV_PROXY} --gas-limit=${GAS} --outfile="deploy-D.json" --chain=D --recall-nonce --send  || return

  TX=$(erdpy data parse --file="deploy-D.json" --expression="data['emitted_tx']['hash']")
  ADDRESS=$(erdpy data parse --file="deploy-D.json" --expression="data['emitted_tx']['address']")

  erdpy data store --key=address-devnet --value=${ADDRESS}
  erdpy data store --key=deployTransaction-devnet --value=${TX}

  echo ""
  echo "Smart contract address: ${ADDRESS}"
}

deposit_tokens() {
  erdpy contract call ${ADDRESS} --pem=${PEM} --proxy=${DEV_PROXY} --gas-limit=${GAS} --chain=D --function=ESDTTransfer --arguments ${MPAD_TICK} ${AMOUNT} ${DEP_ENDPOINT} --send
}

get_ticketEntries() {
  erdpy contract query ${ADDRESS} --proxy=${DEV_PROXY} --function=getTicketRangeForAddress --arguments ${USER_ADDR}
}

get_winningTickets() {
  erdpy contract query ${ADDRESS} --proxy=${DEV_PROXY} --function=getWinningTicketIdsForAddress --arguments ${USER_ADDR}
}

get_confirmedTickets() {
  erdpy contract query ${ADDRESS} --proxy=${DEV_PROXY} --function=getConfirmedTicketIdsForAddress --arguments ${USER_ADDR}
}

addtickets_stage() {
  erdpy contract call ${ADDRESS} --proxy=${DEV_PROXY} --function=setStage --arguments 1
}

selectWinners_stage() {
  erdpy contract call ${ADDRESS} --proxy=${DEV_PROXY} --function=setStage --arguments 2
}

waitConfirmation_stage() {
  erdpy contract call ${ADDRESS} --proxy=${DEV_PROXY} --function=setStage --arguments 3
}

confirmTickets_stage() {
  erdpy contract call ${ADDRESS} --proxy=${DEV_PROXY} --function=setStage --arguments 4
}

selectNewWinners_stage() {
  erdpy contract call ${ADDRESS} --proxy=${DEV_PROXY} --function=setStage --arguments 5
}

waitBeforeClaim_stage() {
  erdpy contract call ${ADDRESS} --proxy=${DEV_PROXY} --function=setStage --arguments 6
}

claim_stage() {
  erdpy contract call ${ADDRESS} --proxy=${DEV_PROXY} --function=setStage --arguments 7
}

