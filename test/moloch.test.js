const nearAPI = require("near-api-js");
const BN = require("bn.js");
const utils = require("./utils");
const { getOrCreateAccount, config } = utils;
require("dotenv").config();

const registerFdai = async (masterAccount, accountId, ftAccountId) => {
  await masterAccount.functionCall({
    contractId: ftAccountId,
    methodName: "storage_deposit",
    args: {
      account_id: accountId
    },
    attachedDeposit: nearAPI.utils.format.parseNearAmount(".01")
  });
};

const registerMoloch = async (masterAccount, accountId, molochAccountId) => {
  console.log(masterAccount);
  console.log(accountId);
  console.log(molochAccountId);
  console.log(nearAPI.utils.format.parseNearAmount(".1"));
  await masterAccount.functionCall({
    contractId: molochAccountId,
    methodName: "storage_deposit",
    args: {
      account_id: accountId,
      registration_only: false
    },
    attachedDeposit: nearAPI.utils.format.parseNearAmount("1")
  });
};

const transferFdai = async (masterAccount, accountId, amount, ftAccountId) => {
  await masterAccount.functionCall({
    contractId: ftAccountId,
    methodName: "ft_transfer",
    args: {
      receiver_id: accountId,
      amount: amount,
      memo: "transfer",
      msg: ""
    },
    attachedDeposit: "1",
    gas: 300000000000000
  });
};

const transferCallFdai = async (
  masterAccount,
  accountId,
  amount,
  ftAccountId
) => {
  await masterAccount.functionCall({
    contractId: ftAccountId,
    methodName: "ft_transfer_call",
    args: {
      receiver_id: accountId,
      amount: amount,
      memo: "transfer",
      msg: ""
    },
    attachedDeposit: "1",
    gas: 300000000000000
  });
};

const balanceOfFdai = async (masterAccount, accountId, ftAccountId) => {
  return await masterAccount.viewFunction(ftAccountId, "ft_balance_of", {
    account_id: accountId
  });
};

const getEscrowBalance = async (masterAccount, accountId, molochAccountId) => {
  return await masterAccount.viewFunction(
    molochAccountId,
    "get_escrow_user_balance",
    { account_id: accountId }
  );
};

const getBankBalance = async (masterAccount, molochAccountId) => {
  return await masterAccount.viewFunction(
    molochAccountId,
    "get_bank_balance",
    {}
  );
};

const getCurentPeriod = async (masterAccount, molochAccountId) => {
  return await masterAccount.viewFunction(
    molochAccountId,
    "get_current_period",
    {}
  );
};

const delay = ms => {
  return new Promise(resolve => setTimeout(resolve, ms));
};

// Setup Contract
// and tear it down
// By deleting the contract account
describe("Moloch test", () => {
  let alice;
  let aliceId;
  let bobId;
  let bob;
  let contractAccount;
  let proposalPeriod;

  const ftAccountId = `${process.env.FDAI_ACCOUNT_ID}.mrkeating.testnet`;
  const masterContractId = process.env.MASTER_ACCOUNT_ID;
  let contractAccountId = `${process.env.MOLOCH_ACCOUNT_ID}.${masterContractId}`;
  const now = Date.now();

  beforeAll(async () => {
    /// some users
    aliceId = "alice-" + now + "." + masterContractId;
    alice = await utils.getOrCreateAccount(aliceId);
    console.log("\n\n Alice accountId:", aliceId, "\n\n");

    bobId = "bob-" + now + "." + masterContractId;
    bob = await getOrCreateAccount(bobId);
    console.log("\n\n Bob accountId:", bobId, "\n\n");

    contractAccount = await getOrCreateAccount(contractAccountId);
    console.log("\n\n contract accountId:", contractAccountId, "\n\n");

    ftAccount = await getOrCreateAccount(ftAccountId);
    console.log("\n\n contract accountId:", ftAccountId, "\n\n");

    masterAccount = await getOrCreateAccount(masterContractId);
    console.log("\n\n master accountId:", masterContractId, "\n\n");

    contract = new nearAPI.Contract(
      contractAccount,
      config.contractName,
      config.contractMethods
    );
    ftContract = new nearAPI.Contract(ftAccount, "mrkeating.testnet", {
      changeMethods: ["ft_transfer", "ft_transfer_call"],
      viewMethods: ["ft_balance_of"]
    });

    // register
    await registerFdai(masterAccount, masterContractId, ftAccountId);
    await registerFdai(masterAccount, aliceId, ftAccountId);
    await registerFdai(masterAccount, contractAccountId, ftAccountId);
    await registerFdai(masterAccount, bobId, ftAccountId);

    await registerMoloch(masterAccount, masterContractId, contractAccountId);
    await registerMoloch(masterAccount, aliceId, contractAccountId);
    await registerMoloch(masterAccount, bobId, contractAccountId);

    await transferFdai(masterAccount, aliceId, "1000", ftAccountId);
    await transferCallFdai(alice, contractAccountId, "100", ftAccountId);
    await transferCallFdai(
      masterAccount,
      contractAccountId,
      "1000",
      ftAccountId
    );
  }, 120000);

  // Create a proposal for bob
  test("Create a proposal for the first member", async () => {
    // proposal_deposit is 10
    //
    // Fetch token balance for master before and after
    // make sure it matches the expected transfer

    // const deposit = new BN(storagePerSale).add(new BN(parseNearAmount('0.1'))).toString();
    await masterAccount.functionCall({
      contractId: contractAccountId,
      methodName: "submit_proposal",
      args: {
        applicant: aliceId,
        token_tribute: "10",
        shares_requested: "10",
        details: "Let's add a second member"
      },
      // attachedDeposit: deposit
      gas: 300000000000000
    });

    proposalPeriod = await getCurentPeriod(masterAccount, contractAccountId);

    // Check balances are correct
    // moloch balance 1100
    const molochBalance = await balanceOfFdai(
      masterAccount,
      contractAccountId,
      ftAccountId
    );
    expect(molochBalance).toEqual("1100");
    // alice balance 900
    const aliceBalance = await balanceOfFdai(
      masterAccount,
      aliceId,
      ftAccountId
    );

    expect(aliceBalance).toEqual("900");
    // Make sure the correct ammount is in escrow
    // Check queue length
    const bankBalance = await getBankBalance(masterAccount, contractAccountId);
    expect(bankBalance).toEqual("0");

    const escrowBalance = await getEscrowBalance(
      masterAccount,
      aliceId,
      contractAccountId
    );
    expect(escrowBalance).toEqual("90");

    const escrowBalanceMaster = await getEscrowBalance(
      masterAccount,
      masterContractId,
      contractAccountId
    );
    expect(escrowBalanceMaster).toEqual("990");
  });

  test("Vote yes on member proposal", async () => {
    // Wait one period
    await delay(1000 * 10);
    await masterAccount.functionCall({
      contractId: contractAccountId,
      methodName: "submit_vote",
      args: {
        proposal_index: "0", // change to 0
        uint_vote: 1
      }
    });

    let vote = await masterAccount.viewFunction(
      contractAccountId,
      "get_member_proposal_vote",
      {
        proposal_index: "0",
        member_id: masterContractId
      }
    );
    console.log("Vote");
    console.log(vote);
    console.log(proposalPeriod);
    expect(vote).toEqual("Yes");
  });

  // Process proposal
  // This can be refactored to send everything to escrow
  // and we can expose a withdrawl mechanism on the main contract
  // rather than having so many cross contract calls
  test("Process passed proposal", async () => {
    // Move move tribute to the guild bank
    // grace period + voting period = 3
    process_period = parseInt(proposalPeriod) + 3;
    console.log(process_period);
    current_period = await getCurentPeriod(masterAccount, contractAccountId);
    console.log(current_period);
    let periodsLeft = process_period - parseInt(current_period);
    if (periodsLeft >= 0) {
      await delay(10000 * (periodsLeft + 1));
    }
    await bob.functionCall({
      contractId: contractAccountId,
      methodName: "process_proposal",
      args: {
        proposal_index: "0"
      },
      attachedDeposit: "1",
      gas: 300000000000000
    });

    const bankBalance = await getBankBalance(masterAccount, contractAccountId);
    expect(bankBalance).toEqual("10");

    const aliceBalance = await balanceOfFdai(
      masterAccount,
      aliceId,
      ftAccountId
    );

    expect(aliceBalance).toEqual("900");

    // Pay processing reward
    const bobBalance = await balanceOfFdai(masterAccount, bobId, ftAccountId);

    expect(bobBalance).toEqual("1");

    // Return proposal deposit
    const escrowBalanceMaster = await getEscrowBalance(
      masterAccount,
      masterContractId,
      contractAccountId
    );
    expect(escrowBalanceMaster).toEqual("999");

    const molochBalance = await balanceOfFdai(
      masterAccount,
      contractAccountId,
      ftAccountId
    );
    expect(molochBalance).toEqual("1099");
  }, 120000);

  // rage quit
  test("Rage quit", async () => {
    await alice.functionCall({
      contractId: contractAccountId,
      methodName: "rage_quit",
      args: {
        shares_to_burn: "5" // 5 of 11 total
      },
      attachedDeposit: "1",
      gas: 300000000000000
    });

    // check the correct amount is withdrawn and sent alice
    const bankBalance = await getBankBalance(masterAccount, contractAccountId);
    // TODO: Double check this rounding is okay
    expect(bankBalance).toEqual("6");
  });

  // abort
  // failed vote
});

// Make sure process proposal and abort assert escrow values
