import { Player } from "./api.js";

let account = "1234";
let player = new Player(account);

async function main() {
  await player.installPlayer();
  let data = await player.getState();

  console.log("player info:");
  console.log(data[0]);

  console.log("object info:");
  console.log(data[1]);

  console.log("global time:");
  console.log(data[2]);

  let config = await player.getConfig();
  console.log("config", config);
}

main();
