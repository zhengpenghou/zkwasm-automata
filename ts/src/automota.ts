import { Player } from "./api.js";

let account = "5678";
let player = new Player(account);

async function main() {
  let config = await player.getConfig();
  console.log("config", config);

  await player.installPlayer();

  await player.deposit();

  await player.installObject([0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n]);

  await player.installObject([0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n]);

  let state = await player.getState();
  console.log("query state:", state);

  await player.deposit();

  await player.withdraw('123456789011121314');
}

main();
