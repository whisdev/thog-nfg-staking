use crate::*;

#[derive(Accounts)]
pub struct WithdrawSOL<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

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


impl WithdrawSOL<'_> {
  
  pub fn process_withdraw_sol(ctx: &mut Context<Self>) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        require!(ctx.accounts.admin.key() == global_authority.super_admin, ThogError::Unautherized);

        // Transfer the SOL rewards to the user
        let _vault_bump = ctx.bumps.reward_vault;
        // Transfer SOL to the winner from the PDA
        sol_transfer_with_signer(
            ctx.accounts.reward_vault.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[&[VAULT_AUTHORITY_SEED.as_ref(), &[_vault_bump]]],
            ctx.accounts.reward_vault.to_account_info().lamports() as u64
        )?;
        Ok(())
    }

}