MOLOCH_ACCOUNT_ID=$(shell grep MOLOCH_ACCOUNT_ID .env | cut -d "=" -f2)

build:
	./build.sh

deploy:
	$(MAKE) build
	./deploy.sh

clean:
	./clean.sh

deploy_contract:
	$(MAKE) build
	near deploy --wasmFile res/moloch.wasm --accountId $(MOLOCH_ACCOUNT_ID).mrkeating.testnet
