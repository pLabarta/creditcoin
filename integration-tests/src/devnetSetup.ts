import { providers, Wallet } from 'creditcoin-js';
import { default as globalSetup } from './globalSetup';

const createWallet = (who: 'lender' | 'borrower') => {
    const lenderPrivateKey = process.env.LENDER_PRIVATE_KEY;
    if (lenderPrivateKey === undefined) {
        throw new Error('LENDER_PRIVATE_KEY environment variable is required');
    }

    const borrowerPrivateKey = process.env.BORROWER_PRIVATE_KEY;
    if (borrowerPrivateKey === undefined) {
        throw new Error('BORROWER_PRIVATE_KEY environment variable is required');
    }

    const privateKey = who === 'lender' ? lenderPrivateKey : borrowerPrivateKey;
    const provider = new providers.JsonRpcProvider((global as any).CREDITCOIN_ETHEREUM_NODE_URL);

    return new Wallet(privateKey, provider);
};

const setup = async () => {
    (global as any).CREDITCOIN_API_URL = 'wss://rpc.devnet.creditcoin.network/ws';
    (global as any).CREDITCOIN_CREATE_WALLET = createWallet;

    (global as any).CREDITCOIN_ETHEREUM_DECREASE_MINING_INTERVAL = false;
    (global as any).CREDITCOIN_ETHEREUM_NAME = 'Rinkeby';
    const ethereumNodeUrl = process.env.ETHEREUM_NODE_URL;
    if (ethereumNodeUrl === undefined) {
        throw new Error('ETHEREUM_NODE_URL environment variable is required');
    }
    (global as any).CREDITCOIN_ETHEREUM_NODE_URL = ethereumNodeUrl;
    (global as any).CREDITCOIN_ETHEREUM_USE_HARDHAT_WALLET = false;

    (global as any).CREDITCOIN_EXECUTE_SETUP_AUTHORITY = false;
    (global as any).CREDITCOIN_NETWORK_LONG_NAME = 'Creditcoin Dev';
    (global as any).CREDITCOIN_NETWORK_SHORT_NAME = 'creditcoin_dev';
    (global as any).CREDITCOIN_METRICS_BASE = 'http://dev-rpc-creditcoin-rpc-2.centralus.cloudapp.azure.com:9615';
    (global as any).CREDITCOIN_REUSE_EXISTING_ADDRESSES = true;

    // This is on Goerli, https://goerli.etherscan.io/address/0x80C9A853B906fc4a30A5F9E4982F1F5AC1798cd0
    (global as any).CREDITCOIN_CTC_CONTRACT_ADDRESS = '0x80C9A853B906fc4a30A5F9E4982F1F5AC1798cd0';
    // we need a new tx hash every time so we call .burn() in globalSetup()! See ctc-deploy.ts
    (global as any).CREDITCOIN_CTC_BURN_TX_HASH = undefined;

    if (process.env.LENDER_PRIVATE_KEY === undefined) {
        throw new Error('LENDER_PRIVATE_KEY environment variable is required');
    }
    (global as any).CREDITCOIN_CTC_DEPLOYER_PRIVATE_KEY = process.env.LENDER_PRIVATE_KEY;

    await globalSetup();
};

export default setup;
