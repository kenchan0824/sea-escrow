use anchor_lang::prelude::*;

declare_id!("HqAjfWXQbRto2wtvCCWr1BNiP1WGKW5fazxbuJrswRmo");

#[program]
pub mod sea_escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
