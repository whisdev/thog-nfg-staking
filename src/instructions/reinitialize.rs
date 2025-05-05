use crate::*;

#[derive(Accounts)]
pub struct ReInitialize<'info> {
  #[account(mut)]
  pub admin: Signer<'info>,
  
  #[account(
    mut,
    seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
    bump,
  )]
  pub global_authority: Account<'info, GlobalPool>,

  #[account(
      mut,
      seeds = [VAULT_AUTHORITY_SEED.as_ref()],
      bump,
  )]
  /// CHECK: This is not dangerous because we don't read or write from this account
  pub reward_vault: AccountInfo<'info>,

  pub system_program: Program<'info, System>,
}

impl ReInitialize<'_> {
  pub fn process_instruction(
    ctx: &mut Context<Self>,
    total_reward_pool: u64,
    daily_drip_percentage: u64
  ) -> Result<()> {
    let global_authority = &mut ctx.accounts.global_authority;
    
    require!(ctx.accounts.admin.key() == global_authority.super_admin, ThogError::Unautherized);

    // global_authority.super_admin = ctx.accounts.admin.key();
    global_authority.total_reward_pool = total_reward_pool * SOLANA_LAMPORTS;
    
    global_authority.daily_drip_percentage = daily_drip_percentage;
    global_authority.daily_drip_amount = (total_reward_pool * SOLANA_LAMPORTS * daily_drip_percentage) / 100 as u64;
    global_authority.reward_per_second = (global_authority.daily_drip_amount / DAY) as u64;

    Ok(())
  }
}
