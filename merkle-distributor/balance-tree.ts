import { UInt } from "@polkadot/types/codec";
import { TypeRegistry } from "@polkadot/types/create";
import { decodeAddress, keccakAsU8a } from "@polkadot/util-crypto";
import MerkleTree from "./merkle-tree";

export default class BalanceTree {
  private readonly tree: MerkleTree;
  constructor(accounts: string[]) {
    this.tree = new MerkleTree(
      accounts.map((account, index) => {
        return BalanceTree.toNode(index, account);
      })
    );
  }

  public getMekleTree(): MerkleTree {
    return this.tree;
  }

  public static verifyProof(
    index: number,
    account: string,
    proof: Buffer[],
    root: Buffer
  ): boolean {
    let pair = BalanceTree.toNode(index, account);
    for (const item of proof) {
      pair = MerkleTree.combinedHash(pair, item);
    }
    return pair.equals(root);
  }

  // keccak256(SCALE.encode(index, account))
  public static toNode(index: number, account: string): Buffer {
    const registry = new TypeRegistry();

    // encode index as u16 SCALE
    let encodedIndex = new UInt(registry, index, 16).toU8a();

    // decode address from ss58 to u8a
    // TODO consider error handling
    let encodedAcc = decodeAddress(account);

    let encodedMsg = new Uint8Array(encodedIndex.length + encodedAcc.length);

    encodedMsg.set(encodedIndex);
    encodedMsg.set(encodedAcc, encodedIndex.length);

    console.log(`Encoded bytes is ${encodedMsg}`);

    let buf = Buffer.from(keccakAsU8a(encodedMsg));
    return buf;
  }

  public getHexRoot(): string {
    return this.tree.getHexRoot();
  }

  // returns the hex bytes32 values of the proof
  public getProof(index: number, account: string): string[] {
    return this.tree.getHexProof(BalanceTree.toNode(index, account));
  }
}
