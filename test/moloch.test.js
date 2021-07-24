const nearAPI = require('near-api-js');
const BN = require('bn.js');
const utils = require('./utils');
const {
  getOrCreateAccount,
	config,
} = utils;
require("dotenv").config();


const registerFdai = async (masterAccount, accountId, ftAccountId) => {
		await masterAccount.functionCall({
			contractId: ftAccountId,
			methodName: 'storage_deposit',
			args: {
				account_id: accountId,
			},
			attachedDeposit: nearAPI.utils.format.parseNearAmount(".01")
		})
}

const transferFdai = async (masterAccount, accountId, amount, ftAccountId) => {
		await masterAccount.functionCall({
			contractId: ftAccountId,
			methodName: 'ft_transfer',
			args: {
				receiver_id: accountId,
				amount: amount,
				memo: "transfer",
				msg: "",
			},
			attachedDeposit: "1",
			gas: 300000000000000
		})


}

const transferCallFdai = async (masterAccount, accountId, amount, ftAccountId) => {
		await masterAccount.functionCall({
			contractId: ftAccountId,
			methodName: 'ft_transfer_call',
			args: {
				receiver_id: accountId,
				amount: amount,
				memo: "transfer",
				msg: "",
			},
			attachedDeposit: "1",
			gas: 300000000000000
		})
}

const balanceOfFdai = async (masterAccount, accountId, ftAccountId) => {
		return await masterAccount.viewFunction(
			ftAccountId,
			'ft_balance_of',
			{account_id: accountId},
		)
}

const getEscrowBalance = async (masterAccount, accountId, molochAccountId) => {
		return await masterAccount.viewFunction(
			molochAccountId,
			'get_escrow_user_balance',
			{account_id: accountId},
		)
}

const getBankBalance = async (masterAccount, molochAccountId) => {
		return await masterAccount.viewFunction(
			molochAccountId,
			'get_bank_balance',
			{},
		)
}





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
	const ftAccountId = "fdai.mrkeating.testnet"
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


		ftAccount = await getOrCreateAccount(ftAccountId);
		console.log('\n\n contract accountId:', ftAccountId, '\n\n');

		masterAccount = await getOrCreateAccount(masterContractId);
		console.log('\n\n master accountId:', masterContractId, '\n\n');



		contract = new nearAPI.Contract(contractAccount, config.contractName, config.contractMethods);
		ftContract = new nearAPI.Contract(ftAccount, "mrkeating.testnet", {changeMethods: ["ft_transfer", "ft_transfer_call"], viewMethods: ["ft_balance_of"]})

		// register
		await registerFdai(masterAccount, masterContractId, ftAccountId)
		await registerFdai(masterAccount, aliceId, ftAccountId)
		await registerFdai(masterAccount, contractAccountId, ftAccountId)
		await transferFdai(masterAccount, aliceId, "1000", ftAccountId)
		await transferCallFdai(alice, contractAccountId, "100", ftAccountId)
		await transferCallFdai(masterAccount, contractAccountId, "1000", ftAccountId)
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
				applicant: aliceId,
				token_tribute: "10",
				shares_requested: "10",
				details: "Let's add a second member",
			},
			// attachedDeposit: deposit
			gas: 300000000000000
		});

		// Check balances are correct
		// moloch balance 1100
		const molochBalance = await balanceOfFdai(masterAccount, contractAccountId, ftAccountId);
		expect(molochBalance).toEqual("1100")
		// alice balance 900
		const aliceBalance = await balanceOfFdai(masterAccount, aliceId, ftAccountId);
		
		expect(aliceBalance).toEqual('900')
		// Make sure the correct ammount is in escrow
		// Check queue length
		const bankBalance = await getBankBalance(masterAccount, contractAccountId)
		expect(bankBalance).toEqual("0")

		const escrowBalance = await getEscrowBalance(masterAccount, aliceId, contractAccountId)
		expect(escrowBalance).toEqual("90")

		const escrowBalanceMaster = await getEscrowBalance(masterAccount, masterContractId, contractAccountId)
		expect(escrowBalanceMaster).toEqual("990")
	})

	// Member votes yes
	// Bob has his proposal added
	// Member processes proposal
	// Then alice and master account vote yes and no
	// get a rage quit

})
