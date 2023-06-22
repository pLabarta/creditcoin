import { Command, OptionValues } from "commander";
import { ApiPromise, BN } from "creditcoin-js";
import { newApi } from "../api";
import {
  checkAddress,
  getCallerSeedFromEnvOrPrompt,
  initECDSAKeyringPairFromPK,
  initKeyringPair,
} from "../utils/account";
import { getBalance, parseCTCString } from "../utils/balance";

export function makeSendCommand() {
  const cmd = new Command("send");
  cmd.description("Send CTC from an account");
  cmd.option(
    "--use-ecdsa",
    "Use ECDSA signature scheme and a private key instead of a mnemonic phrase"
  );
  cmd.option("-a, --amount [amount]", "Amount to send");
  cmd.option("-t, --to [to]", "Specify recipient address");
  cmd.action(sendAction);
  return cmd;
}

async function sendAction(options: OptionValues) {
  const { api } = await newApi(options.url);

  // Check options
  checkAmount(options);
  checkAddress(options.to, api, "send funds to");

  const recipient = options.to;
  const amount = parseCTCString(options.amount);

  const seed = await getCallerSeedFromEnvOrPrompt();
  const caller = options.useEcdsa
    ? initECDSAKeyringPairFromPK(seed)
    : initKeyringPair(seed);

  await checkEnoughFundsToSend(caller.address, amount, api);

  const tx = api.tx.balances.transfer(recipient, amount.toString());

  const hash = await tx.signAndSend(caller);

  console.log("Transfer transaction hash: " + hash.toHex());

  process.exit(0);
}

function checkAmount(options: OptionValues) {
  if (!options.amount) {
    console.log("Must specify amount to send");
    process.exit(1);
  }
}

async function checkEnoughFundsToSend(
  address: string,
  amount: BN,
  api: ApiPromise
) {
  const balance = await getBalance(address, api);
  if (balance.free.sub(balance.miscFrozen).lt(amount)) {
    console.log(
      `Caller ${address} has insufficient funds to send ${amount.toString()}`
    );
    process.exit(1);
  }
}
