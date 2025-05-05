use anchor_lang::prelude::*;
use solana_program::program::{ invoke, invoke_signed };
use anchor_spl::token::{ self };
use crate::*;

// transfer sol from PDA
pub fn sol_transfer_with_signer<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    signers: &[&[&[u8]]; 1],
    amount: u64
) -> Result<()> {
    let ix = solana_program::system_instruction::transfer(source.key, destination.key, amount);
    invoke_signed(&ix, &[source, destination, system_program], signers)?;
    Ok(())
}

//  transfer sol from user
pub fn sol_transfer_user<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    amount: u64
) -> Result<()> {
    let ix = solana_program::system_instruction::transfer(source.key, destination.key, amount);
    invoke(&ix, &[source, destination, system_program])?;
    Ok(())
}

//  transfer token from user
pub fn token_transfer_user<'a>(
    from: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    to: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
    amount: u64
) -> Result<()> {
    let cpi_ctx: CpiContext<_> = CpiContext::new(token_program, token::Transfer {
        from,
        authority,
        to,
    });
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}

//  transfer token from PDA
pub fn token_transfer_with_signer<'a>(
    from: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    to: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
    signers: &[&[&[u8]]; 1],
    amount: u64
) -> Result<()> {
    let cpi_ctx: CpiContext<_> = CpiContext::new_with_signer(
        token_program,
        token::Transfer {
            from,
            authority,
            to,
        },
        signers
    );
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}

pub fn calculate_user_reward<'info>(
    lock_time: u64,
    period: u64,
    staked_amount: u64,
    last_update_time: u64,
    multiplier: u64,
    daily_drip_percentage: u64
) -> u64 {
    let current_timestamp: u64;

    let current_time = Clock::get().unwrap().unix_timestamp as u64;
    if current_time <= lock_time + (period as u64) * DAY {
        current_timestamp = current_time;
    } else {
        current_timestamp = lock_time + (period as u64) * DAY;
    }
    // if unlock_time == 0 {
    // } else {
    //     current_timestamp = unlock_time;
    // }

    let mut time_diff: u64 = 0;
    if current_timestamp >= last_update_time {
        time_diff = current_timestamp - last_update_time;
    }

    let user_weight = (staked_amount * multiplier) / 10000;
    let daily_drip_amount = (user_weight * daily_drip_percentage) / (100 as u64);
    let reward_per_second = (daily_drip_amount / DAY) as u64;

    let accumulated_reward = reward_per_second * time_diff;

    return accumulated_reward;
}

pub fn select_item<'info>(staking_pool: &UserStakingPool, item_id: u64) -> (StakedNFT, usize) {
    let mut temp_nft: StakedNFT = StakedNFT {
        ..Default::default()
    };
    let mut item_index: usize = 0;
    for i in 0..staking_pool.item_count {
        let index = i as usize;
        if staking_pool.items[index].item_id.eq(&item_id) {
            temp_nft = staking_pool.items[index];
            item_index = index;
            break;
        }
    }
    return (temp_nft, item_index);
}
