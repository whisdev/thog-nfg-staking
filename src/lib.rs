use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token, TokenAccount };

pub mod utils;
pub mod constants;
pub mod error;
pub mod instructions;
pub mod account;

use utils::*;
use constants::*;
use error::*;
use instructions::*;
use account::*;

declare_id!("EnEWuQgCQT39BvcRP3wd1qZqEF2Juh5LBj3LcURxQCPo");

#[program]
pub mod thog_nft_staking {
    use super::*;

    pub fn initialize(
        mut ctx: Context<Initialize>,
        total_reward_pool: u64,
        daily_drip_percentage: u64
    ) -> Result<()> {
        Initialize::process_instruction(&mut ctx, total_reward_pool, daily_drip_percentage)
    }

    pub fn reinitialize(
        mut ctx: Context<ReInitialize>,
        total_reward_pool: u64,
        daily_drip_percentage: u64
    ) -> Result<()> {
        ReInitialize::process_instruction(&mut ctx, total_reward_pool, daily_drip_percentage)
    }

    pub fn initialize_user_pool(mut ctx: Context<InitializeUserPool>) -> Result<()> {
        InitializeUserPool::process_instruction(&mut ctx)
    }

    pub fn stake(mut ctx: Context<Stake>, amount: u64, multiplier: u64, period: u64, lock_time: u64) -> Result<()> {
        Stake::stake_thog_nft(&mut ctx, amount, multiplier, period, lock_time)
    }
    
    pub fn stake_thog(mut ctx: Context<StakeThog>, amount: u64, lock_time: u64) -> Result<()> {
        StakeThog::stake_thog_handler(&mut ctx, amount, lock_time)
    }

    pub fn unlock(mut ctx: Context<UnLock>, item_id: u64) -> Result<()> {
        UnLock::unlock_handler(&mut ctx, item_id)
    }

    pub fn unlock_all(mut ctx: Context<UnLock>) -> Result<()> {
        UnLock::unlock_all_handler(&mut ctx)
    }

    pub fn withdraw_thog(mut ctx: Context<WithdrawThog>, item_id: u64) -> Result<()> {
        WithdrawThog::withdraw_thog_handler(&mut ctx, item_id)
    }

    pub fn withdraw_thog_nft(mut ctx: Context<Withdraw>, item_id: u64) -> Result<()> {
        Withdraw::withdraw_thog_nft_handler(&mut ctx, item_id)
    }

    pub fn claim_rewards(mut ctx: Context<ClaimRewards>, item_id: u64) -> Result<()> {
        ClaimRewards::claim_rewards_handler(&mut ctx, item_id)
    }

    pub fn claim_pending_rewards(mut ctx: Context<ClaimRewards>) -> Result<()> {
        ClaimRewards::claim_pending_rewards_handler(&mut ctx)
    }

    pub fn claim_all_rewards(mut ctx: Context<ClaimRewards>) -> Result<()> {
        ClaimRewards::claim_all_rewards_handler(&mut ctx)
    }

    pub fn withdraw_sol(mut ctx: Context<WithdrawSOL>) -> Result<()> {
        WithdrawSOL::process_withdraw_sol(&mut ctx)
    }

    pub fn withdraw_spl(mut ctx: Context<WithdrawSPL>) -> Result<()> {
        WithdrawSPL::process_withdraw_spl(&mut ctx)
    }
}