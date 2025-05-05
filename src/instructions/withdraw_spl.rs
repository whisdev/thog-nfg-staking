use crate::*;

#[derive(Accounts)]
pub struct WithdrawSPL<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    //  admin token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = admin
    )]
    pub ata_admin: Account<'info, TokenAccount>,

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

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl WithdrawSPL<'_> {
    pub fn process_withdraw_spl(ctx: &mut Context<Self>) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        require!(ctx.accounts.admin.key() == global_authority.super_admin, ThogError::Unautherized);

        let balance = ctx.accounts.ata_vault.amount;
        msg!("balance: {:?}", balance);

        let _vault_bump = ctx.bumps.reward_vault;
        // Transfer SPL to the admin from the PDA
        token_transfer_with_signer(
            ctx.accounts.ata_vault.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.ata_admin.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            &[&[VAULT_AUTHORITY_SEED.as_ref(), &[_vault_bump]]],
            balance
        )?;
        Ok(())
    }
}
