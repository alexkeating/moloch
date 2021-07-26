# Make sure that a near account is created and that
MOLOCH_ACCOUNT_ID=$(grep MOLOCH_ACCOUNT_ID .env | cut -d "=" -f2)
FDAI_ACCOUNT_ID=$(grep FDAI_ACCOUNT_ID .env | cut -d "=" -f2)

# a subaccount is made for each subsequent deploy
#
# ten second periods, voting period 20 seconds grace 10 seconds 
near create-account $MOLOCH_ACCOUNT_ID.mrkeating.testnet --master-account mrkeating.testnet
near create-account $FDAI_ACCOUNT_ID.mrkeating.testnet --master-account mrkeating.testnet
near deploy --wasmFile res/test_fungible_token.wasm --accountId $FDAI_ACCOUNT_ID.mrkeating.testnet
near call $FDAI_ACCOUNT_ID.mrkeating.testnet new_default_meta --accountId $FDAI_ACCOUNT_ID.mrkeating.testnet --args '{"owner_id":"mrkeating.testnet","total_supply":"1000000000"}'

near deploy --wasmFile res/moloch.wasm --accountId $MOLOCH_ACCOUNT_ID.mrkeating.testnet
near call $MOLOCH_ACCOUNT_ID.mrkeating.testnet new --accountId $MOLOCH_ACCOUNT_ID.mrkeating.testnet --args '{"summoner": "mrkeating.testnet", "approved_token": "fdaiv4.mrkeating.testnet", "period_duration": "10000000000", "voting_period_length": "2", "grace_period_length": "1", "abort_window": "2", "proposal_deposit": "10", "dilution_bound": "1", "processing_reward": "1"}'
