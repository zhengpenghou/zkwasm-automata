import { Player } from "./api.js";
import { ethers } from "ethers";
import { BigNumber } from '@ethersproject/bignumber';  // Import BigNumber from @ethersproject/bignumber
import abiData from './Proxy.json' assert { type: 'json' };
import dotenv from 'dotenv';

dotenv.config();

let player = new Player(process.env.SERVER_ADMIN_KEY!);
let provider = new ethers.JsonRpcProvider(process.env.RPC_PROVIDER!);

const proxyContract = new ethers.Contract(process.env.SETTLEMENT_CONTRACT_ADDRESS!, abiData.abi, provider);

async function main() {
  proxyContract.on('TopUp', async (l1token: any, address: any, pid_1: bigint, pid_2: bigint, amount: bigint, event: any) => {
    console.log(`TopUp event received: pid_1=${pid_1.toString()}, pid_2=${pid_2.toString()}, amount=${amount.toString()}`);

    // Call the deposit function from the Player instance
    try {
        console.log(typeof(pid_1), typeof(pid_2), typeof(amount));
        await player.deposit(pid_1, pid_2, amount);
        console.log('Deposit successful!');
    } catch (error) {
        console.error('Error during deposit:', error);
    }
  });

  //tbd: Mark 'tx' as handled
}

main();

