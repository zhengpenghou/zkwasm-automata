import {stringify} from "querystring";
import { Player } from "./api.js";
import {ZKWasmAppRpc} from "zkwasm-ts-server";

const rpc = new ZKWasmAppRpc("http://127.0.0.1:3000");
let account = "1234";
let player = new Player(account, rpc);

async function main() {
  await player.installPlayer();
  let data = await player.getState();

  console.log("player info:");
  console.log(JSON.stringify(data));

  let config = await player.getConfig();
  console.log("config", config);
}

main();
