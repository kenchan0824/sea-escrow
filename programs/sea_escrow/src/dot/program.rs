#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use crate::{id, seahorse_util::*};
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::{cell::RefCell, rc::Rc};

#[account]
#[derive(Debug)]
pub struct EscrowOrder {
    pub seller: Pubkey,
    pub order_id: u16,
    pub bump: u8,
    pub seller_token_account: Pubkey,
    pub buyer: Pubkey,
    pub buyer_token_account: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub vault: Pubkey,
    pub state: OrderState,
}

impl<'info, 'entrypoint> EscrowOrder {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedEscrowOrder<'info, 'entrypoint>> {
        let seller = account.seller.clone();
        let order_id = account.order_id;
        let bump = account.bump;
        let seller_token_account = account.seller_token_account.clone();
        let buyer = account.buyer.clone();
        let buyer_token_account = account.buyer_token_account.clone();
        let mint = account.mint.clone();
        let amount = account.amount;
        let vault = account.vault.clone();
        let state = account.state.clone();

        Mutable::new(LoadedEscrowOrder {
            __account__: account,
            __programs__: programs_map,
            seller,
            order_id,
            bump,
            seller_token_account,
            buyer,
            buyer_token_account,
            mint,
            amount,
            vault,
            state,
        })
    }

    pub fn store(loaded: Mutable<LoadedEscrowOrder>) {
        let mut loaded = loaded.borrow_mut();
        let seller = loaded.seller.clone();

        loaded.__account__.seller = seller;

        let order_id = loaded.order_id;

        loaded.__account__.order_id = order_id;

        let bump = loaded.bump;

        loaded.__account__.bump = bump;

        let seller_token_account = loaded.seller_token_account.clone();

        loaded.__account__.seller_token_account = seller_token_account;

        let buyer = loaded.buyer.clone();

        loaded.__account__.buyer = buyer;

        let buyer_token_account = loaded.buyer_token_account.clone();

        loaded.__account__.buyer_token_account = buyer_token_account;

        let mint = loaded.mint.clone();

        loaded.__account__.mint = mint;

        let amount = loaded.amount;

        loaded.__account__.amount = amount;

        let vault = loaded.vault.clone();

        loaded.__account__.vault = vault;

        let state = loaded.state.clone();

        loaded.__account__.state = state;
    }
}

#[derive(Debug)]
pub struct LoadedEscrowOrder<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, EscrowOrder>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub seller: Pubkey,
    pub order_id: u16,
    pub bump: u8,
    pub seller_token_account: Pubkey,
    pub buyer: Pubkey,
    pub buyer_token_account: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub vault: Pubkey,
    pub state: OrderState,
}

#[derive(Clone, Debug, PartialEq, AnchorSerialize, AnchorDeserialize, Copy)]
pub enum OrderState {
    Pending,
    Deposited,
    Dispute,
    Settled,
    Refunded,
}

impl Default for OrderState {
    fn default() -> Self {
        OrderState::Pending
    }
}

pub fn deposit_handler<'info>(
    mut buyer: SeahorseSigner<'info, '_>,
    mut order: Mutable<LoadedEscrowOrder<'info, '_>>,
    mut buyer_token_account: SeahorseAccount<'info, '_, TokenAccount>,
    mut vault: SeahorseAccount<'info, '_, TokenAccount>,
) -> () {
    if !(order.borrow().state == OrderState::Pending) {
        panic!("cannot deposit again");
    }

    if !(vault.key() == order.borrow().vault) {
        panic!("wrong vault inputted");
    }

    token::transfer(
        CpiContext::new(
            buyer_token_account.programs.get("token_program"),
            token::Transfer {
                from: buyer_token_account.to_account_info(),
                authority: buyer.clone().to_account_info(),
                to: vault.clone().to_account_info(),
            },
        ),
        order.borrow().amount.clone(),
    )
    .unwrap();

    assign!(order.borrow_mut().buyer, buyer.key());

    assign!(
        order.borrow_mut().buyer_token_account,
        buyer_token_account.key()
    );

    assign!(order.borrow_mut().state, OrderState::Deposited);
}

pub fn init_order_handler<'info>(
    mut seller: SeahorseSigner<'info, '_>,
    mut seller_token_account: SeahorseAccount<'info, '_, TokenAccount>,
    mut mint: SeahorseAccount<'info, '_, Mint>,
    mut order: Empty<Mutable<LoadedEscrowOrder<'info, '_>>>,
    mut vault: Empty<SeahorseAccount<'info, '_, TokenAccount>>,
    mut order_id: u16,
    mut amount: u64,
) -> () {
    let mut bump = order.bump.unwrap();
    let mut order = order.account.clone();
    let mut vault = vault.account.clone();

    assign!(order.borrow_mut().seller, seller.key());

    assign!(
        order.borrow_mut().seller_token_account,
        seller_token_account.key()
    );

    assign!(order.borrow_mut().mint, mint.key());

    assign!(order.borrow_mut().vault, vault.key());

    assign!(order.borrow_mut().order_id, order_id);

    assign!(order.borrow_mut().amount, amount);

    assign!(order.borrow_mut().bump, bump);

    assign!(order.borrow_mut().state, OrderState::Pending);
}

pub fn release_handler<'info>(
    mut buyer: SeahorseSigner<'info, '_>,
    mut order: Mutable<LoadedEscrowOrder<'info, '_>>,
    mut vault: SeahorseAccount<'info, '_, TokenAccount>,
    mut seller_token_account: SeahorseAccount<'info, '_, TokenAccount>,
) -> () {
    if !(vault.key() == order.borrow().vault) {
        panic!("wrong vault inputted");
    }

    if !(order.borrow().state == OrderState::Deposited) {
        panic!("cannot release before deposit");
    }

    if !(seller_token_account.key() == order.borrow().seller_token_account) {
        panic!("must relase to seller token account");
    }

    let mut seller = order.borrow().seller;
    let mut order_id = order.borrow().order_id;
    let mut bump = order.borrow().bump;

    token::transfer(
        CpiContext::new_with_signer(
            vault.programs.get("token_program"),
            token::Transfer {
                from: vault.to_account_info(),
                authority: order.borrow().__account__.to_account_info(),
                to: seller_token_account.clone().to_account_info(),
            },
            &[Mutable::new(vec![
                "order".to_string().as_bytes().as_ref(),
                seller.as_ref(),
                order_id.to_le_bytes().as_ref(),
                bump.to_le_bytes().as_ref(),
            ])
            .borrow()
            .as_slice()],
        ),
        vault.amount,
    )
    .unwrap();

    assign!(order.borrow_mut().state, OrderState::Settled);
}
