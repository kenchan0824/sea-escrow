import { getAccount } from "@solana/spl-token"
import * as anchor from "@coral-xyz/anchor";
import { web3, Program, BN } from "@coral-xyz/anchor";
import { SeaEscrow } from "../target/types/sea_escrow";
import { SimpleUser } from "@solardev/simple-web3";
const assert = require("assert");

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.SeaEscrow as Program<SeaEscrow>;

describe("Seahorse Escrow - Settle Route", () => {

    let minter: SimpleUser;
    let seller: SimpleUser;
    let buyer: SimpleUser;
    let referee: SimpleUser;

    let orderId: number;
    let orderAddress: web3.PublicKey;
    let vaultAddress: web3.PublicKey;

    before("", async () => {

        minter = await SimpleUser.generate(provider.connection);
        seller = await SimpleUser.generate(provider.connection);
        buyer = await SimpleUser.generate(provider.connection);
        referee = await SimpleUser.generate(provider.connection); 

        await minter.mint("USDC")
            .transfer("USDC", 200, buyer)
            .transfer("USDC", 0, seller)
            .commit();

        orderId = 255;

        [orderAddress, ] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("order"), seller.publicKey.toBuffer(), new BN(orderId).toArrayLike(Buffer, 'le', 2)],
            program.programId
        );

        [vaultAddress,] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault"), orderAddress.toBuffer()],
            program.programId
        );
    });

    it("seller can initiate an escrow order", async () => {

        const amount = new BN(100 * Math.pow(10, 9));
        
        await program.methods.initOrder(orderId, referee.publicKey, amount)
            .accounts({
                seller: seller.publicKey,
                sellerTokenAccount: seller.tokenAccounts["USDC"],
                mint: minter.tokens["USDC"].mint,
                order: orderAddress,
                vault: vaultAddress
            })
            .signers([seller])
            .rpc({ skipPreflight: true });

        const order = await program.account.escrowOrder.fetch(orderAddress);
        assert.ok(order.seller.toBase58() == seller.publicKey.toBase58());
        assert.ok(order.sellerTokenAccount.toBase58() == seller.tokenAccounts["USDC"].toBase58());
        assert.ok(order.mint.toBase58() == minter.tokens["USDC"].mint.toBase58());
        assert.ok(order.amount.toNumber() == 100 * Math.pow(10, 9));
        assert.ok(order.vault.toBase58() == vaultAddress.toBase58());
        assert.ok(order.state.pending);

        const vault = await getAccount(provider.connection, vaultAddress);
        assert.ok(vault.mint.toBase58() == minter.tokens["USDC"].mint.toBase58());
    });

    it("buyer cannot release vault funds before deposit", async () => {
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
                .rpc({ skipPreflight: true });
            
            success = true;
        } catch(err) {}
        
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
            .rpc({ skipPreflight: true });

        const { amount } = await buyer.balance("USDC");
        assert.ok(amount == 100);

        const vault = await getAccount(provider.connection, vaultAddress);
        assert.ok(Number(vault.amount) == 100 * Math.pow(10, 9));

        const order = await program.account.escrowOrder.fetch(orderAddress);
        assert.ok(order.buyer.toBase58() == buyer.publicKey.toBase58());
        assert.ok(order.buyerTokenAccount.toBase58() == buyer.tokenAccounts["USDC"].toBase58());
        assert.ok(order.state.deposited);
    });

    it("buyer cannot deposit repeatedly", async () => {

        let success = false;

        try {
            await program.methods.deposit()
                .accounts({
                    buyer: buyer.publicKey,
                    order: orderAddress,
                    buyerTokenAccount: buyer.tokenAccounts["USDC"],
                    vault: vaultAddress,
                })
                .signers([buyer])
                .rpc({ skipPreflight: true });
            
            success = true;
        } catch(err) {}

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
                .rpc({ skipPreflight: true });
            
            success = true;
        } catch(err) {}

        assert.ok(success == false)
    });

    it("only the buyer can release vault funds", async () => {
        let success = false;
        const hacker = await SimpleUser.generate(provider.connection);
        
        try {
            await program.methods.release()
                .accounts({
                    buyer: hacker.publicKey,
                    order: orderAddress,
                    vault: vaultAddress,
                    sellerTokenAccount: seller.tokenAccounts["USDC"],
                })
                .signers([hacker])
                .rpc({ skipPreflight: true });
            
            success = true;
        } catch(err) {}

        assert.ok(success == false);
    });

    it("buyer can release vault funds", async () => {
    
        await program.methods.release()
            .accounts({
                buyer: buyer.publicKey,
                order: orderAddress,
                vault: vaultAddress,
                sellerTokenAccount: seller.tokenAccounts["USDC"],
            })
            .signers([buyer])
            .rpc({ skipPreflight: true });

        const { amount } = await seller.balance("USDC");
        assert.ok(amount == 100)

        const order = await program.account.escrowOrder.fetch(orderAddress);
        assert.ok(order.state.settled);
    });

    it("buyer cannot dispute after settled", async () => {
        let success = false;

        try {
            await program.methods.dispute()
                .accounts({
                    buyer: buyer.publicKey,
                    order: orderAddress
                })
                .signers([buyer])
                .rpc({ skipPreflight: true });
            
            success = true;
        }
        catch (err) {}

        assert.ok(success == false);
    });
});

describe.skip("Seahorse Escrow - Dispute Route", async () => {

    let minter: SimpleUser;
    let seller: SimpleUser;
    let buyer: SimpleUser;
    let referee: SimpleUser;

    let orderId: number;
    let orderAddress: web3.PublicKey;
    let vaultAddress: web3.PublicKey;

    before("", async () => {

        minter = await SimpleUser.generate(provider.connection);
        seller = await SimpleUser.generate(provider.connection);
        buyer = await SimpleUser.generate(provider.connection);
        referee = await SimpleUser.generate(provider.connection); 

        await minter.mint("USDC")
            .transfer("USDC", 200, buyer)
            .transfer("USDC", 0, seller)
            .commit();

        orderId = 256;

        [orderAddress, ] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("order"), seller.publicKey.toBuffer(), new BN(orderId).toArrayLike(Buffer, 'le', 2)],
            program.programId
        );

        [vaultAddress,] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault"), orderAddress.toBuffer()],
            program.programId
        );

        const amount = new BN(100 * Math.pow(10, 9));
        await program.methods.initOrder(orderId, referee.publicKey, amount)
            .accounts({
                seller: seller.publicKey,
                sellerTokenAccount: seller.tokenAccounts["USDC"],
                mint: minter.tokens["USDC"].mint,
                order: orderAddress,
                vault: vaultAddress
            })
            .signers([seller])
            .rpc({ skipPreflight: true });

        await program.methods.deposit()
            .accounts({
                buyer: buyer.publicKey,
                order: orderAddress,
                buyerTokenAccount: buyer.tokenAccounts["USDC"],
                vault: vaultAddress,
            })
            .signers([buyer])
            .rpc({ skipPreflight: true });
    });

    it("only the buyer can dispute his escort order", async () => {
        let success = false;
        const hacker = await SimpleUser.generate(provider.connection);

        try {
            await program.methods.dispute()
                .accounts({
                    buyer: hacker.publicKey,
                    order: orderAddress
                })
                .signers([hacker])
                .rpc({ skipPreflight: true });
            
            success = true;
        } catch(err) {}
        
        assert.ok(success == false);
    });

    it("buyer can dispute his escort order", async () => {
        await program.methods.dispute()
            .accounts({
                buyer: buyer.publicKey,
                order: orderAddress
            })
            .signers([buyer])
            .rpc({ skipPreflight: true });
            
        const order = await program.account.escrowOrder.fetch(orderAddress);
        assert.ok(order.state.dispute);
    });

    it("referee can refund to the buyer", async() => {
        await program.methods.refund()
            .accounts({
                referee: referee.publicKey,
                order: orderAddress,
                vault: vaultAddress,
                buyerTokenAccount: buyer.tokenAccounts["USDC"]
            })
            .signers([referee])
            .rpc({ skipPreflight: true });

        const order = await program.account.escrowOrder.fetch(orderAddress);
        assert.ok(order.state.refunded);
        
        const { amount } = await buyer.balance("USDC");
        assert.ok(amount == 200);
    })

});
