use {
    crate::*,
    anchor_spl::token::{ Mint, Token, TokenAccount },
    mpl_token_metadata::instructions::{ RevokeStakingV1CpiBuilder, UnlockV1CpiBuilder },
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

    #[account(
        mut, 
        token::mint = nft_token_mint, 
        token::authority = owner,
    )]
    pub nft_token_account: Box<Account<'info, TokenAccount>>,
    pub nft_token_mint: Box<Account<'info, Mint>>,
    /// CHECK instruction will fail if wrong edition is supplied
    pub token_mint_edition: AccountInfo<'info>,
    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub token_mint_record: AccountInfo<'info>,
    /// CHECK instruction will fail if wrong metadata is supplied
    #[account(mut)]
    mint_metadata: UncheckedAccount<'info>,
    /// CHECK instruction will fail if wrong rules are supplied
    pub auth_rules: UncheckedAccount<'info>,
    /// CHECK instruction will fail if wrong sysvar ixns are supplied
    pub sysvar_instructions: AccountInfo<'info>,

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
    /// CHECK intstruction will fail if wrong program is supplied
    pub token_metadata_program: AccountInfo<'info>,
    /// CHECK intstruction will fail if wrong program is supplied
    pub auth_rules_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

impl Withdraw<'_> {
    pub fn withdraw_thog_nft_handler(ctx: &mut Context<Self>, item_id: u64) -> Result<()> {
        let mut staking_pool = ctx.accounts.staking_pool.load_mut()?;
        require!(ctx.accounts.owner.key() == staking_pool.owner, ThogError::OnlyOwnerCanCall);

        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
        let (selected_item, _item_index) = select_item(&staking_pool, item_id);

        require!(
            current_timestamp >= selected_item.unlock_time + (DAY as u64),
            ThogError::WithdrawNotOver
        );

        require!(staking_pool.total_staked_amount > 0, ThogError::InsufficientStakedAmount);

        let _vault_bump = ctx.bumps.reward_vault;
        // if reward > 0 {
        //     // Transfer SOL from the PDA to user
        //     sol_transfer_with_signer(
        //         ctx.accounts.reward_vault.to_account_info(),
        //         ctx.accounts.owner.to_account_info(),
        //         ctx.accounts.system_program.to_account_info(),
        //         &[&[VAULT_AUTHORITY_SEED.as_ref(), &[_vault_bump]]],
        //         reward
        //     )?;
        // }
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

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[ctx.bumps.global_authority]];
        let delegate_seeds = &[&seeds[..]];

        msg!("Start unlock NFT");
        UnlockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
            .authority(&ctx.accounts.global_authority.to_account_info())
            .token_owner(Some(&ctx.accounts.owner))
            .token(&ctx.accounts.nft_token_account.to_account_info())
            .mint(&ctx.accounts.nft_token_mint.to_account_info())
            .metadata(&ctx.accounts.mint_metadata)
            .edition(Some(ctx.accounts.token_mint_edition.as_ref()))
            .token_record(Some(ctx.accounts.token_mint_record.as_ref()))
            .payer(&ctx.accounts.owner)
            .system_program(&ctx.accounts.system_program)
            .sysvar_instructions(&ctx.accounts.sysvar_instructions)
            .spl_token_program(Some(&ctx.accounts.token_program.to_account_info()))
            .authorization_rules_program(Some(ctx.accounts.auth_rules_program.as_ref()))
            .authorization_rules(Some(ctx.accounts.auth_rules.as_ref()))
            .invoke_signed(delegate_seeds)?;

        RevokeStakingV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
            .delegate(&ctx.accounts.global_authority.to_account_info())
            .metadata(&ctx.accounts.mint_metadata.to_account_info())
            .master_edition(Some(ctx.accounts.token_mint_edition.as_ref()))
            .token_record(Some(ctx.accounts.token_mint_record.as_ref()))
            .mint(&ctx.accounts.nft_token_mint.to_account_info())
            .token(&ctx.accounts.nft_token_account.to_account_info())
            .authority(&ctx.accounts.owner)
            .payer(&ctx.accounts.owner)
            .system_program(&ctx.accounts.system_program)
            .sysvar_instructions(&ctx.accounts.sysvar_instructions)
            .spl_token_program(Some(&ctx.accounts.token_program.to_account_info()))
            .authorization_rules_program(Some(ctx.accounts.auth_rules_program.as_ref()))
            .authorization_rules(Some(ctx.accounts.auth_rules.as_ref()))
            .invoke()?;
        msg!("End unlock NFT");

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
