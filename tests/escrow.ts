import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  // TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("escrow", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Escrow as Program<Escrow>;

  const provider = anchor.getProvider();

  const connection = provider.connection;

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  const id = new anchor.BN(1234);
  const id2 = new anchor.BN(1232);

  const [maker, taker, mintA, mintB] = Array.from({ length: 4 }, () =>
    Keypair.generate()
  );

  const [makerAtaA, makerAtaB, takerAtaA, takerAtaB] = [maker, taker]
    .map((a) =>
      [mintA, mintB].map((m) =>
        getAssociatedTokenAddressSync(
          m.publicKey,
          a.publicKey,
          false,
          TOKEN_2022_PROGRAM_ID
        )
      )
    )
    .flat();

  const escrow = PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      maker.publicKey.toBuffer(),
      id.toArrayLike(Buffer, "le", 8),
    ],
    program.programId
  )[0];

  const vault = getAssociatedTokenAddressSync(
    mintA.publicKey,
    escrow,
    true,
    TOKEN_2022_PROGRAM_ID
  );
  const escrow2 = PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      maker.publicKey.toBuffer(),
      id2.toArrayLike(Buffer, "le", 8),
    ],
    program.programId
  )[0];

  const vault2 = getAssociatedTokenAddressSync(
    mintA.publicKey,
    escrow2,
    true,
    TOKEN_2022_PROGRAM_ID
  );
  const accounts = {
    maker: maker.publicKey,
    taker: taker.publicKey,
    mintA: mintA.publicKey,
    mintB: mintB.publicKey,
    makerAtaA,
    makerAtaB,
    takerAtaA,
    takerAtaB,
    escrow,
    vault,
    TOKEN_2022_PROGRAM_ID,
  };

  const tokenProgram = TOKEN_2022_PROGRAM_ID;

  it("airdrop and create mints", async () => {
    let lamports = await getMinimumBalanceForRentExemptMint(connection);
    let tx = new Transaction();
    tx.instructions = [
      ...[maker, taker].map((account) =>
        SystemProgram.transfer({
          fromPubkey: provider.publicKey,
          toPubkey: account.publicKey,
          lamports: 10 * LAMPORTS_PER_SOL,
        })
      ),
      ...[mintA, mintB].map((mint) =>
        SystemProgram.createAccount({
          fromPubkey: provider.publicKey,
          newAccountPubkey: mint.publicKey,
          lamports,
          space: MINT_SIZE,
          programId: tokenProgram,
        })
      ),
      ...[
        { mint: mintA.publicKey, authority: maker.publicKey, ata: makerAtaA },
        { mint: mintB.publicKey, authority: taker.publicKey, ata: takerAtaB },
      ].flatMap((x) => [
        createInitializeMint2Instruction(
          x.mint,
          6,
          x.authority,
          null,
          tokenProgram
        ),
        createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          x.ata,
          x.authority,
          x.mint,
          tokenProgram
        ),
        createMintToInstruction(
          x.mint,
          x.ata,
          x.authority,
          1e9,
          undefined,
          tokenProgram
        ),
      ]),
    ];

    await provider.sendAndConfirm(tx, [mintA, mintB, maker, taker]).then(log);
  });

  it("make", async () => {
    await program.methods
      .make(id, new anchor.BN(1e6), new anchor.BN(1e6))
      .accountsStrict({
        maker: accounts.maker,
        mintA: accounts.mintA,
        mintB: accounts.mintB,
        makerTokenAta: accounts.makerAtaA,
        escrow: accounts.escrow,
        vault: accounts.vault,
        tokenProgram,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([maker])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("refund", async () => {
    await program.methods
      .refund()
      .accountsStrict({
        maker: accounts.maker,
        makerToken: accounts.mintA,
        makerTokenAta: accounts.makerAtaA,
        escrow: accounts.escrow,
        vaultAta: accounts.vault,
        tokenProgram,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("make", async () => {
    await program.methods
      .make(id2, new anchor.BN(1e6), new anchor.BN(1e6))
      .accountsStrict({
        maker: accounts.maker,
        mintA: accounts.mintA,
        mintB: accounts.mintB,
        makerTokenAta: accounts.makerAtaA,
        escrow: escrow2,
        vault: vault2,
        tokenProgram,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([maker])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("take", async () => {
    await program.methods
      .take()
      .accountsStrict({
        maker: accounts.maker,
        taker: accounts.taker,
        makerToken: accounts.mintA,
        takerToken: accounts.mintB,
        escrow: escrow2,
        vaultMintA: vault2,
        makerTokenAtaMintB: accounts.makerAtaB,
        takerTokenAtaMintA: accounts.takerAtaA,
        takerTokenAtaMintB: accounts.takerAtaB,
        tokenProgram,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([taker])
      .rpc()
      .then(confirm)
      .then(log);
  });
});
