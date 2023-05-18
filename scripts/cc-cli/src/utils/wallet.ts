import { Wallet } from "creditcoin-js";

export function initEthereumWallet(privateKey: string) {
  return new Wallet(privateKey);
}
