import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { AnchorNftStaking } from "../target/types/anchor_nft_staking";
import { LootBoxes } from "../target/types/loot_boxes";
import { setupNft } from "./helpers/setup_nft";
import { PROGRAM_ID as METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata';
import { expect } from "chai";
import {
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddress,
  getAccount,
  createMint,
  mintToChecked,
} from "@solana/spl-token-real";
import { BN } from "bn.js";
import {
  promiseWithTimeout,
  SwitchboardTestContext,
} from "@switchboard-xyz/sbv2-utils";
import * as sbv2 from "@switchboard-xyz/switchboard-v2";
import setupSwitchboard from './helpers/setup_switchboard';


describe("anchor-nft-staking", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = anchor.workspace.AnchorNftStaking.provider.wallet;

  const program = anchor.workspace.AnchorNftStaking as Program<AnchorNftStaking>;
  const lootboxProgram = anchor.workspace.LootBoxes as Program<LootBoxes>;

  let stakeStatePda: anchor.web3.PublicKey;
  let nft: any;
  let mint: anchor.web3.PublicKey;
  let tokenAddress: anchor.web3.PublicKey;


  let switchboard: SwitchboardTestContext;
  let userState: anchor.web3.PublicKey;
  let lootboxPointerPda: anchor.web3.PublicKey;
  let permissionBump: number;
  let switchboardStateBump: number;
  let vrfAccount: sbv2.VrfAccount;
  let switchboardStateAccount: sbv2.ProgramStateAccount;
  let permissionAccount: sbv2.PermissionAccount;
  

  before(async () => {
      ({ nft, stakeStatePda, mint, tokenAddress } = await setupNft(
        program,
        wallet.payer
      ));
      ({
          switchboard,
          lootboxPointerPda,
          permissionBump,
          switchboardStateBump,
          vrfAccount,
          switchboardStateAccount,
          permissionAccount,
        } = await setupSwitchboard(provider, lootboxProgram, wallet.payer)
      );
  });

  it("Stake", async () => {
    const tx = await program.methods
      .stake()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        nftMint: nft.mintAddress,
        nftEdition: nft.masterEditionAddress,
        metadataProgram: METADATA_PROGRAM_ID,
      })
      .rpc();

      console.log( `View transaction: https://explorer.solana.com/tx/${tx}?cluster=devnet` );

    const account = await program.account.userStakeInfo.fetch(stakeStatePda);
    expect(account.stakeState === "Staked");
  });

  it("Redeems", async () => {
    const tx = await program.methods
      .redeem()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        stakeMint: mint,
        userStakeAta: tokenAddress,
      })
      .rpc();
    
      console.log(`View transaction: https://explorer.solana.com/tx/${tx}?cluster=devnet`);

    const account = await program.account.userStakeInfo.fetch(stakeStatePda);
    expect(account.stakeState === "Staked");
    const tokenAccount = await getAccount(provider.connection, tokenAddress);
    console.log(tokenAccount.amount);
  });

  it("Unstake", async () => {
    const tx = await program.methods
      .unstake()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        nftMint: nft.mintAddress,
        nftEdition: nft.masterEditionAddress,
        metadataProgram: METADATA_PROGRAM_ID,
        stakeMint: mint,
        userStakeAta: tokenAddress,
      })
      .rpc();
    
      console.log( `View transaction: https://explorer.solana.com/tx/${tx}?cluster=devnet` );

    const account = await program.account.userStakeInfo.fetch(stakeStatePda);
    expect(account.stakeState === "Unstaked");
    const tokenAccount = await getAccount(provider.connection, tokenAddress);
    console.log(tokenAccount.amount);
  });

    it("init user", async () => {
      const tx = await lootboxProgram.methods
        .initUser({
          switchboardStateBump: switchboardStateBump,
          vrfPermissionBump: permissionBump,
        })
        .accounts({
          state: userState,
          vrf: vrfAccount.publicKey,
          payer: wallet.pubkey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    });

    
  it("Chooses a random lootbox",async () => {

    const [stakeAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [wallet.publicKey.toBuffer(), nft.tokenAddress.toBuffer()],
      program.programId
    );

    const vrfState = await vrfAccount.loadData();
    const { authority, dataBuffer } = await switchboard.queue.loadData();

    await lootboxProgram.methods
      .openLootbox(new BN(10))
      .accounts({
        user: wallet.publicKey,
        stakeMint: mint,
        stakeMintAta: tokenAddress,
        stakeState: stakeAccount,
        state: userState,
        vrf: vrfAccount.publicKey,
        oracleQueue: switchboard.queue.publicKey,
        queueAuthority: authority,
        dataBuffer: dataBuffer,
        permission: permissionAccount.publicKey,
        escrow: vrfState.escrow,
        programState: switchboardStateAccount.publicKey,
        switchboardProgram: switchboard.program.programId,
        payerWallet: switchboard.payerTokenWallet,
        recentBlockhashes: anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
      })
      .rpc();

    await awaitCallback(
      lootboxProgram,
      lootboxPointerPda,
      20_000,
      "Didn't get random mint"
    );

    const [address] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lootbox"), wallet.publicKey.toBuffer()],
      lootboxProgram.programId
    );
    const pointer = await lootboxProgram.account.lootboxPointer.fetch(address);

    expect(pointer.isInitialized == true);
    expect(pointer.redeemable == true);

  })

   it("Mints the selected gear", async () => {
    
     const [pointerAddress] = anchor.web3.PublicKey.findProgramAddressSync(
       [Buffer.from("lootbox"), wallet.publicKey.toBuffer()],
       lootboxProgram.programId
     );

     const pointer = await lootboxProgram.account.lootboxPointer.fetch(
       pointerAddress
     );

     let previousGearCount = 0;
     const gearAta = await getAssociatedTokenAddress(
       pointer.mint,
       wallet.publicKey
     );
     try {
       let gearAccount = await getAccount(provider.connection, gearAta);
       previousGearCount = Number(gearAccount.amount);
     } catch (error) {}

     await lootboxProgram.methods
       .retrieveItemFromLootbox()
       .accounts({
         mint: pointer.mint,
         userGearAta: gearAta,
       })
       .rpc();

     const gearAccount = await getAccount(provider.connection, gearAta);
     expect(Number(gearAccount.amount)).to.equal(previousGearCount + 1);
   });

});

async function awaitCallback(
  program: Program<LootBoxes>,
  lootboxPointerAddress: anchor.web3.PublicKey,
  timeoutInterval: number,
  errorMsg = "Timed out waiting for VRF Client callback"
) {
  let ws: number | undefined = undefined;
  const result: boolean = await promiseWithTimeout(
    timeoutInterval,
    new Promise((resolve: (result: boolean) => void) => {
      ws = program.provider.connection.onAccountChange(
        lootboxPointerAddress,
        async (
          accountInfo: anchor.web3.AccountInfo<Buffer>,
          context: anchor.web3.Context
        ) => {
          const lootboxPointer = await program.account.lootboxPointer.fetch(
            lootboxPointerAddress
          );

          if (lootboxPointer.redeemable) {
            resolve(true);
          }
        }
      );
    }).finally(async () => {
      if (ws) {
        await program.provider.connection.removeAccountChangeListener(ws);
      }
      ws = undefined;
    }),
    new Error(errorMsg)
  ).finally(async () => {
    if (ws) {
      await program.provider.connection.removeAccountChangeListener(ws);
    }
    ws = undefined;
  });

  return result;
}