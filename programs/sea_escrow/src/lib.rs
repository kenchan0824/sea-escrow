#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub mod dot;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use dot::program::*;
use std::{cell::RefCell, rc::Rc};

declare_id!("HqAjfWXQbRto2wtvCCWr1BNiP1WGKW5fazxbuJrswRmo");

pub mod seahorse_util {
    use super::*;
    use std::{
        collections::HashMap,
        fmt::Debug,
        ops::{Deref, Index, IndexMut},
    };

    pub struct Mutable<T>(Rc<RefCell<T>>);

    impl<T> Mutable<T> {
        pub fn new(obj: T) -> Self {
            Self(Rc::new(RefCell::new(obj)))
        }
    }

    impl<T> Clone for Mutable<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Deref for Mutable<T> {
        type Target = Rc<RefCell<T>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Debug> Debug for Mutable<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: Default> Default for Mutable<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    pub trait IndexWrapped {
        type Output;

        fn index_wrapped(&self, index: i128) -> &Self::Output;
    }

    pub trait IndexWrappedMut: IndexWrapped {
        fn index_wrapped_mut(&mut self, index: i128) -> &mut <Self as IndexWrapped>::Output;
    }

    impl<T> IndexWrapped for Vec<T> {
        type Output = T;

        fn index_wrapped(&self, mut index: i128) -> &Self::Output {
            if index < 0 {
                index += self.len() as i128;
            }

            let index: usize = index.try_into().unwrap();

            self.index(index)
        }
    }

    impl<T> IndexWrappedMut for Vec<T> {
        fn index_wrapped_mut(&mut self, mut index: i128) -> &mut <Self as IndexWrapped>::Output {
            if index < 0 {
                index += self.len() as i128;
            }

            let index: usize = index.try_into().unwrap();

            self.index_mut(index)
        }
    }

    impl<T, const N: usize> IndexWrapped for [T; N] {
        type Output = T;

        fn index_wrapped(&self, mut index: i128) -> &Self::Output {
            if index < 0 {
                index += N as i128;
            }

            let index: usize = index.try_into().unwrap();

            self.index(index)
        }
    }

    impl<T, const N: usize> IndexWrappedMut for [T; N] {
        fn index_wrapped_mut(&mut self, mut index: i128) -> &mut <Self as IndexWrapped>::Output {
            if index < 0 {
                index += N as i128;
            }

            let index: usize = index.try_into().unwrap();

            self.index_mut(index)
        }
    }

    #[derive(Clone)]
    pub struct Empty<T: Clone> {
        pub account: T,
        pub bump: Option<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramsMap<'info>(pub HashMap<&'static str, AccountInfo<'info>>);

    impl<'info> ProgramsMap<'info> {
        pub fn get(&self, name: &'static str) -> AccountInfo<'info> {
            self.0.get(name).unwrap().clone()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithPrograms<'info, 'entrypoint, A> {
        pub account: &'entrypoint A,
        pub programs: &'entrypoint ProgramsMap<'info>,
    }

    impl<'info, 'entrypoint, A> Deref for WithPrograms<'info, 'entrypoint, A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            &self.account
        }
    }

    pub type SeahorseAccount<'info, 'entrypoint, A> =
        WithPrograms<'info, 'entrypoint, Box<Account<'info, A>>>;

    pub type SeahorseSigner<'info, 'entrypoint> = WithPrograms<'info, 'entrypoint, Signer<'info>>;

    #[derive(Clone, Debug)]
    pub struct CpiAccount<'info> {
        #[doc = "CHECK: CpiAccounts temporarily store AccountInfos."]
        pub account_info: AccountInfo<'info>,
        pub is_writable: bool,
        pub is_signer: bool,
        pub seeds: Option<Vec<Vec<u8>>>,
    }

    #[macro_export]
    macro_rules! seahorse_const {
        ($ name : ident , $ value : expr) => {
            macro_rules! $name {
                () => {
                    $value
                };
            }

            pub(crate) use $name;
        };
    }

    pub trait Loadable {
        type Loaded;

        fn load(stored: Self) -> Self::Loaded;

        fn store(loaded: Self::Loaded) -> Self;
    }

    macro_rules! Loaded {
        ($ name : ty) => {
            <$name as Loadable>::Loaded
        };
    }

    pub(crate) use Loaded;

    #[macro_export]
    macro_rules! assign {
        ($ lval : expr , $ rval : expr) => {{
            let temp = $rval;

            $lval = temp;
        }};
    }

    #[macro_export]
    macro_rules! index_assign {
        ($ lval : expr , $ idx : expr , $ rval : expr) => {
            let temp_rval = $rval;
            let temp_idx = $idx;

            $lval[temp_idx] = temp_rval;
        };
    }

    pub(crate) use assign;

    pub(crate) use index_assign;

    pub(crate) use seahorse_const;
}

#[program]
mod sea_escrow {
    use super::*;
    use seahorse_util::*;
    use std::collections::HashMap;

    #[derive(Accounts)]
    pub struct Deposit<'info> {
        #[account(mut)]
        pub buyer: Signer<'info>,
        #[account(mut)]
        pub order: Box<Account<'info, dot::program::EscrowOrder>>,
        #[account(mut)]
        pub buyer_token_account: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub vault: Box<Account<'info, TokenAccount>>,
        pub token_program: Program<'info, Token>,
    }

    pub fn deposit(ctx: Context<Deposit>) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let buyer = SeahorseSigner {
            account: &ctx.accounts.buyer,
            programs: &programs_map,
        };

        let order = dot::program::EscrowOrder::load(&mut ctx.accounts.order, &programs_map);
        let buyer_token_account = SeahorseAccount {
            account: &ctx.accounts.buyer_token_account,
            programs: &programs_map,
        };

        let vault = SeahorseAccount {
            account: &ctx.accounts.vault,
            programs: &programs_map,
        };

        deposit_handler(
            buyer.clone(),
            order.clone(),
            buyer_token_account.clone(),
            vault.clone(),
        );

        dot::program::EscrowOrder::store(order);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (order_id : u16 , amount : u64)]
    pub struct InitOrder<'info> {
        #[account(mut)]
        pub seller: Signer<'info>,
        #[account(mut)]
        pub seller_token_account: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub mint: Box<Account<'info, Mint>>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: EscrowOrder > () + 8 , payer = seller , seeds = ["order" . as_bytes () . as_ref () , seller . key () . as_ref () , order_id . to_le_bytes () . as_ref ()] , bump)]
        pub order: Box<Account<'info, dot::program::EscrowOrder>>,
        # [account (init , payer = seller , seeds = ["vault" . as_bytes () . as_ref () , order . key () . as_ref ()] , bump , token :: mint = mint , token :: authority = order)]
        pub vault: Box<Account<'info, TokenAccount>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
    }

    pub fn init_order(ctx: Context<InitOrder>, order_id: u16, amount: u64) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let seller = SeahorseSigner {
            account: &ctx.accounts.seller,
            programs: &programs_map,
        };

        let seller_token_account = SeahorseAccount {
            account: &ctx.accounts.seller_token_account,
            programs: &programs_map,
        };

        let mint = SeahorseAccount {
            account: &ctx.accounts.mint,
            programs: &programs_map,
        };

        let order = Empty {
            account: dot::program::EscrowOrder::load(&mut ctx.accounts.order, &programs_map),
            bump: Some(ctx.bumps.order),
        };

        let vault = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.vault,
                programs: &programs_map,
            },
            bump: Some(ctx.bumps.vault),
        };

        init_order_handler(
            seller.clone(),
            seller_token_account.clone(),
            mint.clone(),
            order.clone(),
            vault.clone(),
            order_id,
            amount,
        );

        dot::program::EscrowOrder::store(order.account);

        return Ok(());
    }

    #[derive(Accounts)]
    pub struct Release<'info> {
        #[account(mut)]
        pub buyer: Signer<'info>,
        #[account(mut)]
        pub order: Box<Account<'info, dot::program::EscrowOrder>>,
        #[account(mut)]
        pub vault: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub seller_token_account: Box<Account<'info, TokenAccount>>,
    }

    pub fn release(ctx: Context<Release>) -> Result<()> {
        let mut programs = HashMap::new();
        let programs_map = ProgramsMap(programs);
        let buyer = SeahorseSigner {
            account: &ctx.accounts.buyer,
            programs: &programs_map,
        };

        let order = dot::program::EscrowOrder::load(&mut ctx.accounts.order, &programs_map);
        let vault = SeahorseAccount {
            account: &ctx.accounts.vault,
            programs: &programs_map,
        };

        let seller_token_account = SeahorseAccount {
            account: &ctx.accounts.seller_token_account,
            programs: &programs_map,
        };

        release_handler(
            buyer.clone(),
            order.clone(),
            vault.clone(),
            seller_token_account.clone(),
        );

        dot::program::EscrowOrder::store(order);

        return Ok(());
    }
}
