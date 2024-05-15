import { getAccount } from "@solana/spl-token"
import * as anchor from "@coral-xyz/anchor";
import { web3, Program, BN } from "@coral-xyz/anchor";
import { SeaEscrow } from "../target/types/sea_escrow";
import { SimpleUser } from "@solardev/simple-web3";
const assert = require("assert");

describe("Seahorse Escrow", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    let minter: SimpleUser;
    let seller: SimpleUser;
    let buyer: SimpleUser;

    let orderId: number;
    let orderAddress: web3.PublicKey;
    let vaultAddress: web3.PublicKey;

    const program = anchor.workspace.SeaEscrow as Program<SeaEscrow>;

    before("", async () => {

        minter = await SimpleUser.generate(provider.connection);
        seller = await SimpleUser.generate(provider.connection);
        buyer = await SimpleUser.generate(provider.connection);

        await minter.mint("USDC")
            .transfer("USDC", 100, buyer)
            .transfer("USDC", 0, seller)
            .commit();

        orderId = 255;

        [orderAddress,] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("order"), seller.publicKey.toBuffer(), new BN(orderId).toArrayLike(Buffer, 'le', 2)],
            program.programId
        );

        [vaultAddress,] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault"), orderAddress.toBuffer()],
            program.programId
        );
    });

    it("seller can initiate an escrow order", async () => {

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

        const order = await program.account.escrowOrder.fetch(orderAddress);
        assert.ok(order.seller.toBase58() == seller.publicKey.toBase58());
        assert.ok(order.sellerTokenAccount.toBase58() == seller.tokenAccounts["USDC"].toBase58());
        assert.ok(order.mint.toBase58() == minter.tokens["USDC"].toBase58());
        assert.ok(order.amount.toNumber() == 100_000_000_000);
        assert.ok(order.vault.toBase58() == vaultAddress.toBase58());
        assert.ok(order.state.pending);

        const vault = await getAccount(provider.connection, vaultAddress);
        assert.ok(vault.mint.toBase58() == minter.tokens["USDC"].toBase58());
    });

    it("buyer cannot release fund before deposit", async () => {
        let success = false;
        
        try {
            await program.methods.release()
                .accounts({
                    buyer: buyer.publicKey,
                    order: orderAddress,
                    vault: vaultAddress,
                    sellerTokenAccount: seller.tokenAccounts["USDC"],
                })
                .signers([buyer])
                .rpc();
            
            success = true;
        } catch(err) {
            success = false;
        }
        assert.ok(success == false)
    });

    it("buyer can deposit to the order vault", async () => {

        await program.methods.deposit()
            .accounts({
                buyer: buyer.publicKey,
                order: orderAddress,
                buyerTokenAccount: buyer.tokenAccounts["USDC"],
                vault: vaultAddress,
            })
            .signers([buyer])
            .rpc();

        const balance = await buyer.balance("USDC");
        assert.ok(balance == 0);
        const vault = await getAccount(provider.connection, vaultAddress);
        assert.ok(Number(vault.amount) == 100_000_000_000);

        const order = await program.account.escrowOrder.fetch(orderAddress);
        assert.ok(order.buyer.toBase58() == buyer.publicKey.toBase58());
        assert.ok(order.buyerTokenAccount.toBase58() == buyer.tokenAccounts["USDC"].toBase58());
        assert.ok(order.state.deposited);
    });

    it("buyer cannot deposit repeatedly", async () => {

        let success = false;
        await minter.transfer("USDC", 100, buyer).commit();

        try {
            await program.methods.deposit()
                .accounts({
                    buyer: buyer.publicKey,
                    order: orderAddress,
                    buyerTokenAccount: buyer.tokenAccounts["USDC"],
                    vault: vaultAddress,
                })
                .signers([buyer])
                .rpc();
            
            success = true;
        } catch(err) {
            success = false;
        }
        assert.ok(success == false)
    });

    it("buyer must release to the seller token account", async () => {
        let success = false;
        
        try {
            await program.methods.release()
                .accounts({
                    buyer: buyer.publicKey,
                    order: orderAddress,
                    vault: vaultAddress,
                    sellerTokenAccount: buyer.tokenAccounts["USDC"],
                })
                .signers([buyer])
                .rpc();
            
            success = true;
        } catch(err) {
            success = false;
        }
        assert.ok(success == false)
    });

    // it("buyer can release vault fund", async () => {
    
    // });

});
