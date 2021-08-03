const fs = require("fs");
const BN = require("bn.js");
const nearAPI = require("near-api-js");
const {
  keyStores: { InMemoryKeyStore },
  Near,
  Account,
  Contract,
  KeyPair,
  utils: {
    format: { parseNearAmount }
  }
} = nearAPI;

const config = {
  explorerUrl: "https://explorer.testnet.near.org",
  networkId: "testnet",
  nodeUrl: "https://rpc.testnet.near.org",
  // walletUrl: 'http://localhost:1234',
  walletUrl: "https://wallet.testnet.near.org",
  helperUrl: "https://helper.testnet.near.org",
  GAS: "200000000000000",
  DEFAULT_NEW_ACCOUNT_AMOUNT: ".5",
  DEFAULT_NEW_CONTRACT_AMOUNT: "5",
  contractName: "mrkeating.testnet",
  contractMethods: {
    changeMethods: [
      "submit_proposal",
      "send_applicant_tribute",
      "submit_vote",
      "process_proposal",
      "rage_quit",
      "abort",
      "update_delegate_key"
    ],
    viewMethods: [
      "get_current_period",
      "get_member_proposal_vote",
      "has_voting_expired"
    ]
  }
};

console.log(
  "Loading Credentials:\n",
  `${process.env.HOME}/.near-credentials/${config.networkId}/${config.contractName}.json`
);

const serializedCreds = fs
  .readFileSync(
    `${process.env.HOME}/.near-credentials/${config.networkId}/${config.contractName}.json`
  )
  .toString();
const credentials = JSON.parse(serializedCreds);
const keyStore = new InMemoryKeyStore();
keyStore.setKey(
  config.networkId,
  config.contractName,
  KeyPair.fromString(credentials.private_key)
);

const near = new Near({
  networkId: config.networkId,
  nodeUrl: config.nodeUrl,
  deps: { keyStore }
});
const { connection } = near;

function generateUniqueSubAccount() {
  return `t${Date.now()}.${config.contractName}`;
}

async function createAccount(
  accountId,
  fundingAmount = config.DEFAULT_NEW_ACCOUNT_AMOUNT,
  secret
) {
  const contractAccount = new Account(connection, config.contractName);
  const newKeyPair = secret
    ? KeyPair.fromString(secret)
    : KeyPair.fromRandom("ed25519");
  await contractAccount.createAccount(
    accountId,
    newKeyPair.publicKey,
    new BN(parseNearAmount(fundingAmount))
  );
  keyStore.setKey(config.networkId, accountId, newKeyPair);
  return new nearAPI.Account(connection, accountId);
}

async function getOrCreateAccount(
  accountId,
  fundingAmount = config.DEFAULT_NEW_ACCOUNT_AMOUNT
) {
  accountId = accountId || generateUniqueSubAccount();
  const account = new nearAPI.Account(connection, accountId);
  try {
    await account.state();
    return account;
  } catch (e) {
    if (!/does not exist/.test(e.toString())) {
      throw e;
    }
  }
  return await createAccount(accountId, fundingAmount);
}

module.exports = {
  getOrCreateAccount,
  connection,
  config
};
