MOLOCH_ACCOUNT_ID=$(shell grep MOLOCH_ACCOUNT_ID .env | cut -d "=" -f2)

build:
	./scripts/build.sh

deploy:
	$(MAKE) build
	./scripts/deploy.sh

clean:
	./scripts/clean.sh

deploy_contract:
	$(MAKE) build
	near deploy --wasmFile res/moloch.wasm --accountId $(MOLOCH_ACCOUNT_ID).mrkeating.testnet

unit:
	cd contracts && cargo test --all

end_to_end:
	$(MAKE) deploy
	yarn run test
	$(MAKE) clean
