use crate::*;

#[derive(Accounts)]
pub struct UnLock<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub staking_pool: AccountLoader<'info, UserStakingPool>,
}

impl UnLock<'_> {
    pub fn unlock_handler(ctx: &mut Context<Self>, item_id: u64) -> Result<()> {

        let mut staking_pool = ctx.accounts.staking_pool.load_mut()?;
        require!(ctx.accounts.owner.key() == staking_pool.owner, ThogError::OnlyOwnerCanCall);

        let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
        let (selected_item, item_index) = select_item(&staking_pool, item_id);

        require!(
            selected_item.unlock_time == 0 as u64,
            ThogError::AlreadyUnlocked
        );

        // msg!("Lock period:: {} days", selected_item.period);

        // require!(
        //     current_timestamp >= selected_item.lock_time + selected_item.period as u64 * DAY,
        //     ThogError::BeforeLockTime
        // );

        staking_pool.items[item_index].unlock_time = current_timestamp;
        
        Ok(())
    }

    pub fn unlock_all_handler(ctx: &mut Context<Self>) -> Result<()> {

        let mut staking_pool = ctx.accounts.staking_pool.load_mut()?;
        require!(ctx.accounts.owner.key() == staking_pool.owner, ThogError::OnlyOwnerCanCall);

        for i in 0..staking_pool.item_count {
            let index = i as usize;
            
            if staking_pool.items[index].unlock_time > 0 {
                continue;
            }
            
            let current_timestamp = Clock::get().unwrap().unix_timestamp as u64;
            // if current_timestamp >= staking_pool.items[index].lock_time + staking_pool.items[index].period as u64 * DAY {
            //     continue;
            // }
    
            staking_pool.items[index].unlock_time = current_timestamp;
        }
        
        Ok(())
    }
}