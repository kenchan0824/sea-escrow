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
    Resolved = 5

class EscrowOrder(Account):
    seller: Pubkey
    order_id: u16
    bump: u8
    seller_token_account: Pubkey
    referee: Pubkey
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
    referee: Pubkey, 
    amount: u64,
):
    bump = order.bump()
    
    order = order.init(
        payer = seller,
        seeds = ["order", seller, order_id]
    )
    vault = vault.init(
        payer = seller,
        seeds = ["vault", order.key()],
        mint = mint,
        authority = order,    
    )

    order.seller = seller.key()
    order.seller_token_account = seller_token_account.key()
    order.mint = mint.key()
    order.vault = vault.key()
    order.order_id = order_id
    order.referee = referee
    order.amount = amount
    order.bump = bump
    order.state = OrderState.Pending


@instruction
def deposit(
    buyer: Signer,
    order: EscrowOrder,
    buyer_token_account: TokenAccount,
    vault: TokenAccount
):
    assert order.state == OrderState.Pending, "cannot deposit again"
    assert vault.key() == order.vault, "wrong vault inputted"
    
    buyer_token_account.transfer(
        authority = buyer,
        to = vault,
        amount = order.amount,
    )
    order.buyer = buyer.key()
    order.buyer_token_account = buyer_token_account.key()
    order.state = OrderState.Deposited

    
@instruction
def release(
    buyer: Signer,
    order: EscrowOrder,
    vault: TokenAccount,
    seller_token_account: TokenAccount
):
    assert vault.key() == order.vault, "wrong vault inputted"
    assert order.state == OrderState.Deposited, "cannot release before deposit"
    assert seller_token_account.key() == order.seller_token_account, "must relase to seller token account"
    assert buyer.key() == order.buyer, "not your escrow order"
    
    seller = order.seller
    order_id = order.order_id
    bump = order.bump
    
    vault.transfer(
        to = seller_token_account,
        amount = vault.amount(),
        authority = order,
        signer = ["order", seller, order_id, bump]
    )
    
    order.state = OrderState.Settled


@instruction
def dispute(
    buyer: Signer,
    order: EscrowOrder,
):
    assert buyer.key() == order.buyer, "not your escrow order"
    assert order.state == OrderState.Deposited, "not deposited or settled"
    
    order.state = OrderState.Dispute
    
    
@instruction
def refund(
    referee: Signer,
    order: EscrowOrder,
    vault: TokenAccount,
    buyer_token_account: TokenAccount
):
    assert vault.key() == order.vault, "wrong vault inputted"
    assert order.state == OrderState.Dispute, "cannot refund before dispute"
    assert buyer_token_account.key() == order.buyer_token_account, "must relase to buyer token account"
    assert referee.key() == order.referee, "you are not referee"
    
    seller = order.seller
    order_id = order.order_id
    bump = order.bump
    
    vault.transfer(
        to = buyer_token_account,
        amount = vault.amount(),
        authority = order,
        signer = ["order", seller, order_id, bump]
    )
    
    order.state = OrderState.Refunded
