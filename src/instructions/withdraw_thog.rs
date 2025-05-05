use { 
    crate::*, 
    anchor_spl::token::{ Mint, Token, TokenAccount }
};

#[derive(Accounts)]
pub struct WithdrawThog<'info> {
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

    //  user token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = owner
    )]
    pub ata_user: Account<'info, TokenAccount>,

    pub token_mint: Account<'info, Mint>,

    //  reward_vault token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = reward_vault,
    )]
    pub ata_vault: Box<Account<'info, TokenAccount>>,

    //  reward_vault
    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub reward_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl WithdrawThog<'_> {
    pub fn withdraw_thog_handler(ctx: &mut Context<Self>, item_id: u64) -> Result<()> {
        let mut staking_pool = ctx.accounts.staking_pool.load_mut()?;
        require!(ctx.accounts.owner.key() == staking_pool.owner, ThogError::OnlyOwnerCanCall);

        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
        let (selected_item, _item_index) = select_item(&staking_pool, item_id);

        require!(
            current_timestamp >= selected_item.lock_time + DAY * (selected_item.period as u64),
            ThogError::WithdrawAfter
        );

        require!(staking_pool.total_staked_amount > 0, ThogError::InsufficientStakedAmount);

        let _vault_bump = ctx.bumps.reward_vault;

        if selected_item.staked_amount > 0 {
            // Transfer thog token to the user
            token_transfer_with_signer(
                ctx.accounts.ata_vault.to_account_info(),
                ctx.accounts.reward_vault.to_account_info(),
                ctx.accounts.ata_user.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                &[&[VAULT_AUTHORITY_SEED.as_ref(), &[_vault_bump]]],
                selected_item.staked_amount as u64
            )?;
        }

        staking_pool.total_staked_amount -= selected_item.staked_amount;
        let user_weight =
            ((staking_pool.total_staked_amount as u64) * selected_item.multiplier) / 10000;
        let global_authority = &mut ctx.accounts.global_authority;
        global_authority.total_staked -= selected_item.staked_amount as u64;
        global_authority.total_weight -= user_weight;
        let reward: u64 = staking_pool.remove_nft(
            ctx.accounts.owner.key(),
            selected_item.item_id,
            global_authority.daily_drip_percentage
        )?;
        staking_pool.pending_reward += reward as u64;

        Ok(())
    }
}
