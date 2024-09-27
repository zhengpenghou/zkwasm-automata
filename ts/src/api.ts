import { query, LeHexBN, ZKWasmAppRpc } from "zkwasm-ts-server";
import BN from "bn.js";

/* The modifier mush less than eight */
function encode_modifier(modifiers: Array<bigint>) {
  let c = 0n;
  for (const m of modifiers) {
    c = (c << 8n) + m;
  }
  return c;
}

function bytesToHex(bytes: Array<number>): string  {
    return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
}

function addrToParams(bn: BN): Array<bigint> {
  // address is encoded in BigEndian
  const mask = new BN('ffffffffffffffff', 16);
  let a = bn.and(mask).toArray('le', 8);
  let b = bn.shrn(64).and(mask).toArray('le', 8);
  let c = bn.shrn(128).and(mask).toArray('le', 8);
  let aHex = a.map(byte => byte.toString(16).padStart(2, '0')).join('');
  let bHex = b.map(byte => byte.toString(16).padStart(2, '0')).join('');
  let cHex = c.map(byte => byte.toString(16).padStart(2, '0')).join('');
  return [BigInt(`0x${cHex}`), BigInt(`0x${bHex}`), BigInt(`0x${aHex}`)];
}

const CMD_INSTALL_PLAYER = 1n;
const CMD_INSTALL_OBJECT = 2n;
const CMD_RESTART_OBJECT = 3n;
const CMD_UPGRADE_OBJECT = 4n;
const CMD_INSTALL_CARD = 5n;
const CMD_WITHDRAW= 6n;
const CMD_DEPOSIT = 7n;

function createCommand(nonce: bigint, command: bigint, objindex: bigint) {
  return (nonce << 16n) + (objindex << 8n) + command;
}

const rpc = new ZKWasmAppRpc("http://localhost:3000");

export class Player {
  processingKey: string;
  constructor(key: string) {
    this.processingKey = key
  }

  async getConfig(): Promise<any> {
    let config = await rpc.query_config();
    return config;
  }

  async getState(): Promise<any> {
    // Get the state response
    let state = await rpc.queryState(this.processingKey);

    // Parse the response to ensure it is a plain JSON object
    const parsedState = JSON.parse(JSON.stringify(state));

    // Extract the data from the parsed response
    const data = JSON.parse(parsedState.data);

    return data;
  }

  async getNonce(): Promise<bigint> {
    const data = await this.getState();
    let nonce = BigInt(data.player.nonce);
    return nonce;
  }

  async installPlayer() {
    try {
      let finished = await rpc.sendTransaction(
        new BigUint64Array([createCommand(0n, CMD_INSTALL_PLAYER, 0n), 0n, 0n, 0n]),
        this.processingKey
      );
      console.log("installPlayer processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("installPlayer error at processing key:", this.processingKey);
    }
  }

  async installObject(objid: bigint, modifiers: Array<bigint>) {
    let nonce = await this.getNonce();
    try {
      let finished = await rpc.sendTransaction(
        new BigUint64Array([createCommand(nonce, CMD_INSTALL_OBJECT, objid), encode_modifier(modifiers), 0n, 0n]),
        this.processingKey
      );
      console.log("installObject processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("installObject error at modifiers:", modifiers, "processing key:", this.processingKey);
    }
  }

  async restartObject(objid: bigint, modifiers: Array<bigint>) {
    let nonce = await this.getNonce();
    try {
      let finished = await rpc.sendTransaction(
        new BigUint64Array([createCommand(nonce, CMD_RESTART_OBJECT, objid), encode_modifier(modifiers), 0n, 0n]),
        this.processingKey
      );
      console.log("restartObject processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e);
        console.log(e.message);
      }
      console.log("restart object error", "processing key:", this.processingKey);
    }
  }

  async upgradeObject(objid: bigint) {
    let nonce = await this.getNonce();
    try {
      let finished = await rpc.sendTransaction(
        new BigUint64Array([createCommand(nonce, CMD_UPGRADE_OBJECT, objid), 0n, 0n, 0n]),
        this.processingKey
      );
      console.log("upgradeObject processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("upgrade object error", "processing key:", this.processingKey);
    }
  }



  async installCard() {
    let nonce = await this.getNonce();
    try {
      let finished = await rpc.sendTransaction(
        new BigUint64Array([createCommand(nonce, CMD_INSTALL_CARD, 0n), 0n, 0n, 0n]),
        this.processingKey
      );
      console.log("installCard processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("installCard error with processing key:", this.processingKey);
    }
  }

  async deposit() {
    let nonce = await this.getNonce();
    let accountInfo = new LeHexBN(query(this.processingKey).pkx).toU64Array();
    try {
      let finished = await rpc.sendTransaction(
        new BigUint64Array([createCommand(nonce, CMD_DEPOSIT, 0n), accountInfo[1], accountInfo[2], 1000n]),
        this.processingKey
      );
      console.log("deposit processed at:", finished);
    } catch(e) {
      if(e instanceof Error) {
        console.log(e.message);
      }
      console.log("deposit error at processing key:", this.processingKey);
    }
  }

  async withdrawRewards(address: string, amount: bigint) {
    let nonce = await this.getNonce();
    let addressBN = new BN(address, 16);
    let a = addressBN.toArray("be", 20); // 20 bytes = 160 bits and split into 4, 8, 8

    console.log("address is", address);
    console.log("address be is", a);


    /*
  (32 bit amount | 32 bit highbit of address)
  (64 bit mid bit of address (be))
  (64 bit tail bit of address (be))
     */


    let firstLimb = BigInt('0x' + bytesToHex(a.slice(0,4).reverse()));
    let sndLimb = BigInt('0x' + bytesToHex(a.slice(4,12).reverse()));
    let thirdLimb = BigInt('0x' + bytesToHex(a.slice(12, 20).reverse()));


    console.log("first is", firstLimb);
    console.log("snd is", sndLimb);
    console.log("third is", thirdLimb);

    try {
      let processStamp = await rpc.sendTransaction(
        new BigUint64Array([
          createCommand(nonce, CMD_WITHDRAW, 0n),
          (firstLimb << 32n) + amount,
          sndLimb,
          thirdLimb
        ]), this.processingKey);
      console.log("withdraw rewards processed at:", processStamp);
    } catch(e) {
      if (e instanceof Error) {
        console.log(e.message);
      }
      console.log("collect reward error at address:", address);
    }
  }
}
