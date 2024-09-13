import { Player } from "./api.js";

let account = "1234";
let player = new Player(account);

async function main() {
  let config = await player.getConfig();
  console.log("config", config);

  await player.installPlayer();
  await player.deposit();
  await player.installObject(0n, [0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n]);
  await player.installCard();
  //await player.restartObject(0n, [0n, 0n, 0n, 0n, 0n, 0n, 0n, 4n]);
  await player.upgradeObject(0n);
  let state = await player.getState();
  console.log("query state:", state);

  await player.deposit();

  await player.withdrawRewards("c177d1d314C8FFe1Ea93Ca1e147ea3BE0ee3E470", 1n);
}

main();
