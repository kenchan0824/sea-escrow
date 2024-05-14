import { getAccount } from "@solana/spl-token"
import * as anchor from "@coral-xyz/anchor";
import { web3, Program, BN } from "@coral-xyz/anchor";
import { SeaEscrow } from "../target/types/sea_escrow";
import { SimpleUser } from "@solardev/simple-web3";
const assert = require("assert");

describe("sea_escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let minter = undefined;
  let seller = undefined;
  let buyer = undefined;

  const program = anchor.workspace.SeaEscrow as Program<SeaEscrow>;

  it("seller can initiate an escrow account", async () => {
    minter = await SimpleUser.generate(provider.connection);
    seller = await SimpleUser.generate(provider.connection);
    buyer = await SimpleUser.generate(provider.connection);

    await minter.mint("USDC")
      .transfer("USDC", 100, buyer)
      .transfer("USDC", 0, seller)
      .commit();

    const orderId = 255;

    const [orderAddress] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("order"), seller.publicKey.toBuffer(), new BN(orderId).toArrayLike(Buffer, 'le', 2)],
      program.programId
    )

    const [vaultAddress] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), orderAddress.toBuffer()],
      program.programId
    );

    await program.methods.initOrder(orderId, new BN(100_000_000_000))
      .accounts({
        seller: seller.publicKey,
        sellerTokenAccount: seller.tokenAccounts["USDC"],
        mint: minter.tokens["USDC"],
        order: orderAddress,
        vault: vaultAddress
      })
      .signers([seller])
      .rpc();

    const order = await program.account.order.fetch(orderAddress);
    assert.ok(order.seller.toBase58() == seller.publicKey.toBase58());
    assert.ok(order.sellerTokenAccount.toBase58() == seller.tokenAccounts["USDC"].toBase58());
    assert.ok(order.mint.toBase58() == minter.tokens["USDC"].toBase58());
    assert.ok(order.amount.toNumber() == 100_000_000_000);

    const tokenAccount = await getAccount(provider.connection, vaultAddress)
    assert.ok(tokenAccount.mint.toBase58() == minter.tokens["USDC"].toBase58());
  });
});
