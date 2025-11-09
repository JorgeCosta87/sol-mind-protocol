import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolMindProtocol } from "../target/types/sol_mind_protocol";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

describe("sol-mind-protocol", () => {
  const provider = anchor.AnchorProvider.env(); 
  anchor.setProvider(provider);

  const program = anchor.workspace.solMindProtocol as Program<SolMindProtocol>;

  const owner = anchor.web3.Keypair.fromSeed(
    Uint8Array.from(Buffer.from("CQB35H179dvx8ADpLUiWkf45XWfydGMF"))
  );

  const collectionKeypair = new anchor.web3.Keypair();
  const treasuryPubkey = new anchor.web3.Keypair().publicKey;


  const collectionConfigPDA = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("collection_config"), collectionKeypair.publicKey.toBuffer()
    ],
    program.programId
  )[0];

  const collection_name = "Teste collection".toString();
  const collection_uri = "teste".toString();
  const collectionSupply = new anchor.BN(100);
  const collectionMintPrice = new anchor.BN(1_000_000_000);
  const collectionRoyalties = 450;

  it("Create collection", async () => {
    const tx = await program.methods
    .createCollection(
      collection_name,
      collection_uri,
      collectionMintPrice,
      collectionRoyalties,
      collectionSupply,
    )
    .accountsPartial({
      owner: owner.publicKey,
      collection: collectionKeypair.publicKey,
      collectionConfig: collectionConfigPDA,
      treasury: treasuryPubkey,
      systemProgram: SYSTEM_PROGRAM_ID
    })
    .signers([owner, collectionKeypair])
    .rpc();
    console.log("Your transaction signature", tx);
  });
});
