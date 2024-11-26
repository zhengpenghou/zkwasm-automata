import { Player } from "./api.js";
import { ethers } from "ethers";
import { BigNumber } from '@ethersproject/bignumber';  // Import BigNumber from @ethersproject/bignumber
import abiData from './Proxy.json' assert { type: 'json' };
import dotenv from 'dotenv';
import {ZKWasmAppRpc} from "zkwasm-ts-server";

dotenv.config();

const rpc = new ZKWasmAppRpc("https://disco.0xrobot.cx:8085");

let player = new Player(process.env.SERVER_ADMIN_KEY!, rpc);
let provider = new ethers.JsonRpcProvider(process.env.RPC_PROVIDER!);

const proxyContract = new ethers.Contract(process.env.SETTLEMENT_CONTRACT_ADDRESS!, abiData.abi, provider);

async function main() {
  proxyContract.on('TopUp', async (l1token: any, address: any, pid_1: bigint, pid_2: bigint, amount: bigint, event: any) => {
    console.log(`TopUp event received: pid_1=${pid_1.toString()}, pid_2=${pid_2.toString()}, amount=${amount.toString()} wei`) ;


    //amount in wei to ether
    let amountInEther = amount / BigInt(10**18);
    console.log("deposited amount (in ether): ", amountInEther);

    if (amountInEther < 1n) {
    	console.error("amount must at least 1 Titan(in ether instead of wei)");
    }else{
      // Call the deposit function from the Player instance
      try {
        console.log(typeof(pid_1), typeof(pid_2), typeof(amount), typeof(amountInEther));
        await player.deposit(pid_1, pid_2, amountInEther);
        console.log('Deposit successful!');
      } catch (error) {
        console.error('Error during deposit:', error);
      }
    }
  });

  //tbd: Mark 'tx' as handled
}

main();
