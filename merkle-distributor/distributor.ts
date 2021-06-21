import BalanceTree from './balance-tree'
import { Keyring } from "@polkadot/api";
import { cryptoWaitReady } from "@polkadot/util-crypto"

//await cryptoWaitReady()

//const keyring = new Keyring({ type: "sr25519" })

const alice = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';//keyring.addFromUri("//Alice", { name: "Alice default" });
const bob = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';//keyring.addFromUri("//Bob", { name: "Bob default" });
//const dave = '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy';

console.log(`Start building a merkle tree ...`);

let tree = new BalanceTree([
  alice,
  bob,
])

console.log(`Build a tree from ${tree.getMekleTree().getLeaves()}`);

console.log(`Merkle tree built successfully!!!`);

console.log(`Root is ${tree.getHexRoot()}`);

console.log(`Proof of Alice is ${tree.getProof(0, alice)}`);