import { Player } from "./api.js";

let account = "1234";
let player = new Player(account);

async function main() {
  let config = await player.getConfig();
  console.log("config", config);

  console.log("install player ...\n");
  await player.installPlayer();

  console.log("deposit ...\n");
  await player.deposit();

  console.log("install object ...\n");
  await player.installObject(0n, [0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n]);

  console.log("install card...\n");
  await player.installCard();

  console.log("restart object ...\n");
  await player.restartObject(0n, [0n, 0n, 0n, 0n, 0n, 0n, 0n, 4n]);

  console.log("upgrade object ...\n");
  await player.upgradeObject(0n);

  let state = await player.getState();
  console.log("query state:", state);

  await player.deposit();

  console.log("withdraw:\n");
  await player.withdrawRewards("c177d1d314C8FFe1Ea93Ca1e147ea3BE0ee3E470", 1n);
}

main();
