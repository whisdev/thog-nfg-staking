use crate::*;

#[derive(Accounts)]
pub struct InitializeUserPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(zero)]
    pub staking_pool: AccountLoader<'info, UserStakingPool>,

    pub system_program: Program<'info, System>,
}

impl InitializeUserPool<'_> {
  pub fn process_instruction(ctx: &mut Context<Self>) -> Result<()> {
    let mut staking_pool = ctx.accounts.staking_pool.load_init()?;
    staking_pool.owner = ctx.accounts.owner.key();
  
    Ok(())
  }
}