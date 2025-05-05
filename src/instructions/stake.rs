use {
    crate::*, 
    anchor_spl::token::{Mint, Token, TokenAccount},
    mpl_token_metadata::{
        // accounts::Metadata, instructions::{DelegateStakingV1CpiBuilder, LockV1CpiBuilder}, 
        instructions::{DelegateStakingV1CpiBuilder, LockV1CpiBuilder}, 
    }, 
};

#[derive(Accounts)]
pub struct Stake<'info> {
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

    token_program: Program<'info, Token>,
    /// CHECK intstruction will fail if wrong program is supplied
    token_metadata_program: AccountInfo<'info>,
    /// CHECK intstruction will fail if wrong program is supplied
    auth_rules_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>
}

impl Stake<'_> {
  pub fn stake_thog_nft(ctx: &mut Context<Self>, amount: u64, multiplier: u64, period: u64, lock_time: u64) -> Result<()> {
    // // Verify metadata is legit
    // let nft_metadata = Metadata::safe_deserialize(&mut ctx.accounts.mint_metadata.to_account_info().data.borrow_mut()).unwrap();
    
    // // Check if this NFT is the wanted collection and verified
    // if let Some(creators) = nft_metadata.creators {
    //     let mut valid: u8 = 0;
    //     for creator in creators {
    //         if creator.address.to_string() == COLLECTION_ADDRESS {
    //             valid = 1;
    //             break;
    //         }
    //     }
    //     require!(valid == 1, ThogError::InvalidCollection);
    // } else {
    //     return Err(error!(ThogError::MetadataCreatorParseError));
    // };
    
    let seeds = &[
        GLOBAL_AUTHORITY_SEED.as_bytes(), 
        &[ctx.bumps.global_authority]
    ];
    let delegate_seeds = &[&seeds[..]];

    msg!("Start Lock NFT");
    DelegateStakingV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
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
        .amount(1)
        // .authorization_data(Some(None))
        .invoke()?;

    LockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
        .authority(&ctx.accounts.global_authority.to_account_info().clone())
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
    msg!("End Lock NFT");

    msg!("Start transfer token to this pda");
    // Transfer amount thog token to this PDA
    token_transfer_user(
        ctx.accounts.ata_user.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.ata_vault.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount
    )?;
    msg!("end transfer token to this pda");

    let mut staking_pool = ctx.accounts.staking_pool.load_mut()?;
    staking_pool.total_staked_amount += amount;
    
    let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
    let staked_item = StakedNFT {
        nft_addr: ctx.accounts.nft_token_account.key(),
        staked_amount: amount,
        multiplier,
        period,
        lock_time: current_timestamp,
        item_id: lock_time,
        unlock_time: 0 as u64,
        last_update_time: current_timestamp,
    };
    staking_pool.add_nft(staked_item);

    let global_authority = &mut ctx.accounts.global_authority;
    global_authority.total_staked += amount;
    global_authority.total_weight += staking_pool.total_staked_amount * multiplier / 10000;
  
    Ok(())
  }
  
}