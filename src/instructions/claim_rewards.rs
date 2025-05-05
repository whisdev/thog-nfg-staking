use crate::*;

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub staking_pool: AccountLoader<'info, UserStakingPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    //  reward_vault
    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub reward_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl ClaimRewards<'_> {
  pub fn claim_rewards_handler(ctx: &mut Context<Self>, item_id: u64) -> Result<()> {
    // Transfer the SOL rewards to the user
    let _vault_bump = ctx.bumps.reward_vault;
    let mut staking_pool = ctx.accounts.staking_pool.load_mut()?;
    
    require!(ctx.accounts.owner.key() == staking_pool.owner, ThogError::OnlyOwnerCanCall);
    
    let (selected_item, item_index) = select_item(&staking_pool, item_id);
    
    let global_authority = &ctx.accounts.global_authority;

    msg!("Calc amounts, staked amount: {}, last updated time: {}, multiplier: {}, reward per second: {}, total weight: {}", selected_item.staked_amount, selected_item.last_update_time, selected_item.multiplier, global_authority.reward_per_second, global_authority.total_weight);

    let reward = calculate_user_reward(
        selected_item.lock_time,
        selected_item.period,
        selected_item.staked_amount,
        selected_item.last_update_time,
        selected_item.multiplier,
        global_authority.daily_drip_percentage
    );

    msg!("Requesting Reward Amount: {} SOL", reward);

    require!(reward > 0, ThogError::InsufficientClaimAmount);

    require!(
        ctx.accounts.reward_vault.to_account_info().lamports() > reward,
        ThogError::InsufficientRewardVaultBalance
    );
  
    // Transfer SOL to the winner from the PDA
    sol_transfer_with_signer(
        ctx.accounts.reward_vault.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        &[&[VAULT_AUTHORITY_SEED.as_ref(), &[_vault_bump]]],
        reward
    )?;

    let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
    staking_pool.items[item_index].last_update_time = current_timestamp;
    Ok(())
  }

  pub fn claim_pending_rewards_handler(ctx: &mut Context<Self>) -> Result<()> {
    // Transfer the SOL rewards to the user
    let _vault_bump = ctx.bumps.reward_vault;
    let mut staking_pool = ctx.accounts.staking_pool.load_mut()?;
  
    require!(ctx.accounts.owner.key() == staking_pool.owner, ThogError::OnlyOwnerCanCall);
    let claim_amount: u64 = staking_pool.pending_reward;

    require!(claim_amount > 0, ThogError::InsufficientClaimAmount);

    require!(
        ctx.accounts.reward_vault.to_account_info().lamports() > claim_amount,
        ThogError::InsufficientRewardVaultBalance
    );
  
    // Transfer SOL to the winner from the PDA
    sol_transfer_with_signer(
        ctx.accounts.reward_vault.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        &[&[VAULT_AUTHORITY_SEED.as_ref(), &[_vault_bump]]],
        claim_amount
    )?;
    staking_pool.pending_reward = 0 as u64;
    
    Ok(())
  }
  
  pub fn claim_all_rewards_handler(ctx: &mut Context<Self>) -> Result<()> {
    // Transfer the SOL rewards to the user
    let _vault_bump = ctx.bumps.reward_vault;
    let mut staking_pool = ctx.accounts.staking_pool.load_mut()?;

    // Check if the owner of the staking pool matches the caller
    require!(ctx.accounts.owner.key() == staking_pool.owner, ThogError::OnlyOwnerCanCall);
    let mut claim_amount: u64 = 0;
    msg!("claim all rewards handler");
    let global_authority = &ctx.accounts.global_authority;

    // Ensure the staking pool items count is valid
    require!(staking_pool.item_count as usize <= staking_pool.items.len(), ThogError::InvalidItemCount);

    for i in 0..staking_pool.item_count {
        let index = i as usize;

        let reward = calculate_user_reward(
            staking_pool.items[index].lock_time,
            staking_pool.items[index].period,
            staking_pool.items[index].staked_amount,
            staking_pool.items[index].last_update_time,
            staking_pool.items[index].multiplier,
            global_authority.daily_drip_percentage
        );

        msg!("Per rewards index = {}, calculated reward: {}", index, reward);
        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
        staking_pool.items[index].last_update_time = current_timestamp;
        claim_amount += reward;
    }

    claim_amount += staking_pool.pending_reward;

    // Ensure the claim amount is greater than zero
    require!(claim_amount > 0, ThogError::InsufficientClaimAmount);

    // Ensure the reward vault has enough lamports to cover the claim amount
    require!(
        ctx.accounts.reward_vault.to_account_info().lamports() > claim_amount,
        ThogError::InsufficientRewardVaultBalance
    );

    // Transfer SOL to the winner from the PDA
    sol_transfer_with_signer(
        ctx.accounts.reward_vault.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        &[&[VAULT_AUTHORITY_SEED.as_ref(), &[_vault_bump]]],
        claim_amount
    )?;

    // Reset the pending reward in the staking pool
    staking_pool.pending_reward = 0;

    Ok(())
}

}
