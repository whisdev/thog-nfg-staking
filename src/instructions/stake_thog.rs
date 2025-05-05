use {
  crate::*, 
  anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct StakeThog<'info> {
  #[account(mut)]
  pub admin: Signer<'info>,

  #[account(mut)]
  pub owner: Signer<'info>,

  #[account(
      mut,
      seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
      bump,
  )]
  pub global_authority: Account<'info, GlobalPool>,

  #[account(mut)]
  pub staking_pool: AccountLoader<'info, UserStakingPool>,

  //  user token account
  #[account(
      mut,
      token::mint = token_mint,
      token::authority = owner
  )]
  pub ata_user: Account<'info, TokenAccount>,

  pub token_mint: Account<'info, Mint>,

  //  reward_vault
  #[account(
      mut,
      seeds = [VAULT_AUTHORITY_SEED.as_ref()],
      bump,
  )]
  /// CHECK: This is not dangerous because we don't read or write from this account
  pub reward_vault: AccountInfo<'info>,

  //  reward_vault token account
  #[account(
      mut,
      token::mint = token_mint,
      token::authority = reward_vault,
  )]
  pub ata_vault: Box<Account<'info, TokenAccount>>,

  token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>
}

impl StakeThog<'_> {
  pub fn stake_thog_handler(ctx: &mut Context<Self>, amount: u64, lock_time: u64) -> Result<()> {
    // Transfer amount thog token to this PDA
    token_transfer_user(
        ctx.accounts.ata_user.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.ata_vault.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount
    )?;
    let mut staking_pool = ctx.accounts.staking_pool.load_mut()?;
    staking_pool.total_staked_amount += amount;
    
    let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
    let staked_item = StakedNFT {
        nft_addr: Pubkey::default(),
        staked_amount: amount,
        multiplier: 10000 as u64,
        period: 30 as u64,
        lock_time: current_timestamp as u64,
        item_id: lock_time,
        unlock_time: 0 as u64,
        last_update_time: current_timestamp as u64,
    };
    staking_pool.add_nft(staked_item);

    let global_authority = &mut ctx.accounts.global_authority;
    global_authority.total_staked += amount;
    global_authority.total_weight += staking_pool.total_staked_amount;

    Ok(())
  }
}
