MOLOCH_ACCOUNT_ID=$(grep MOLOCH_ACCOUNT_ID .env | cut -d "=" -f2)
FDAI_ACCOUNT_ID=$(grep FDAI_ACCOUNT_ID .env | cut -d "=" -f2)

near delete $MOLOCH_ACCOUNT_ID.mrkeating.testnet mrkeating.testnet
near delete $FDAI_ACCOUNT_ID.mrkeating.testnet mrkeating.testnet

