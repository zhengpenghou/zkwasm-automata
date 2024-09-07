import {stringify} from "querystring";
import { Player } from "./api.js";

let account = "1234";
let player = new Player(account);

async function main() {
  await player.installPlayer();
  let data = await player.getState();

  console.log("player info:");
  console.log(JSON.stringify(data));

  let config = await player.getConfig();
  console.log("config", config);
}

main();
