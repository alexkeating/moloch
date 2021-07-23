const nearAPI = require('near-api-js');
const BN = require('bn.js');
const utils = require('./utils');
const {
  getOrCreateAccount,
	config,
} = utils;
require("dotenv").config();


// Setup Contract
// and tear it down 
// By deleting the contract account
describe('Moloch test', () => {
	let alice
	let aliceId
	let bob
	let bobId
	let contract
	let contractAccount
	let ftContract
	const masterContractId = process.env.MASTER_ACCOUNT_ID
	let contractAccountId = `${process.env.MOLOCH_ACCOUNT_ID}.${masterContractId}`
  const now = Date.now();

	beforeAll(async () => {
		

		/// some users
		aliceId = 'alice-' + now + '.' + masterContractId;
		alice = await utils.getOrCreateAccount(aliceId);
		console.log('\n\n Alice accountId:', aliceId, '\n\n');

		bobId = 'bob-' + now + '.' + masterContractId;
		bob = await getOrCreateAccount(bobId);
		console.log('\n\n Bob accountId:', bobId, '\n\n');

		contractAccount = await getOrCreateAccount(contractAccountId);
		console.log('\n\n contract accountId:', contractAccountId, '\n\n');

		masterAccount = await getOrCreateAccount(masterContractId);
		console.log('\n\n master accountId:', masterContractId, '\n\n');



		contract = new nearAPI.Contract(contractAccount, config.contractName, config.contractMethods);
		ftContract = new nearAPI.Contract("boop.testnet", "boo.testnet", {changeMethods: ["ft_transfer"], viewMethods: ["ft_balance_of"]})

	})

	// Create a proposal for bob
	test('Create a proposal for the first member', async () => {
		// proposal_deposit is 10
		//
		// Fetch token balance for master before and after
		// make sure it matches the expected transfer

				// const deposit = new BN(storagePerSale).add(new BN(parseNearAmount('0.1'))).toString();
		await masterAccount.functionCall({
			contractId: contractAccountId,
			methodName: 'submit_proposal',
			args: {
				applicant: bobId,
				token_tribute: "10",
				shares_requested: "10",
				details: "Let's add a second member",
			},
			// attachedDeposit: deposit
			gas: 300000000000000
		});

		// we can check by getting the balance
		console.log("Here")
		console.log(contractAccount)

	})

})
