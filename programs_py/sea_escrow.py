# sea_escrow
# Built with Seahorse v0.2.0

from seahorse.prelude import *

declare_id('HqAjfWXQbRto2wtvCCWr1BNiP1WGKW5fazxbuJrswRmo')

class OrderState(Enum):
    Pending = 0
    Deposited = 1
    Dispute = 2
    Settled = 3
    Refunded = 4

class EscrowOrder(Account):
    seller: Pubkey
    seller_token_account: Pubkey
    buyer: Pubkey
    buyer_token_account: Pubkey
    mint: Pubkey
    amount: u64
    vault: Pubkey
    state: OrderState

@instruction
def init_order(
    seller: Signer,
    seller_token_account: TokenAccount,
    mint: TokenMint,
    order: Empty[EscrowOrder],
    vault: Empty[TokenAccount],
    order_id: u16,
    amount: u64
):
    order = order.init(
        payer = seller,
        seeds = ['order', seller, order_id]
    )
    order.seller = seller.key()
    order.seller_token_account = seller_token_account.key()
    order.mint = mint.key()
    order.amount = amount
    order.state = OrderState.Pending

    vault = vault.init(
        payer = seller,
        seeds = ['vault', order.key()],
        mint = mint,
        authority = order,    
    )

@instruction
def deposit(
    buyer: Signer,
    order: EscrowOrder,
    buyer_token_account: TokenAccount,
    vault: TokenAccount,
    amount: u64
):
    assert order.state == OrderState.Pending, "cannot deposit again"
    assert amount >= order.amount, "amount must be enough"
    
    buyer_token_account.transfer(
        authority = buyer,
        to = vault,
        amount = amount,
    )
    order.buyer = buyer.key()
    order.buyer_token_account = buyer_token_account.key()
    order.state = OrderState.Deposited