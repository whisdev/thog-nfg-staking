use anchor_lang::prelude::*;
use crate::*;
#[account]
#[derive(Default)]
pub struct GlobalPool {
    pub super_admin: Pubkey,        // 32
    pub total_reward_pool: u64,     // 8
    pub daily_drip_percentage: u64, // 8
    pub total_staked: u64,          // 8
    pub total_weight: u64,          // 8
    pub daily_drip_amount: u64,     // 8
    pub reward_per_second: u64,     // 8
}
#[zero_copy]
#[derive(Default)]
pub struct StakedNFT {
    pub nft_addr: Pubkey,                   // 32
    pub staked_amount: u64,                 // 8
    pub period: u64,                        // 8
    pub multiplier: u64,                    // 8
    pub lock_time: u64,                     // 8
    pub item_id: u64,                       // 8
    pub unlock_time: u64,                   // 8
    pub last_update_time: u64,              // 8
}
#[account(zero_copy)]
pub struct UserStakingPool {
    pub owner: Pubkey,                      // 32
    pub item_count: u64,                    // 8
    pub items: [StakedNFT; NFT_NUMBER],     // 88 * 30 = 2640
    pub total_staked_amount: u64,           // 8
    pub pending_reward: u64,                // 8
}

impl Default for UserStakingPool {
    #[inline]
    fn default() -> UserStakingPool {
        UserStakingPool {
            owner: Pubkey::default(),
            item_count: 0,
            items: [StakedNFT {
                ..Default::default()
            }; NFT_NUMBER],
            total_staked_amount: 0,
            pending_reward: 0,
        }
    }
}

impl UserStakingPool {
    pub const DATA_SIZE: usize = 8 + 2696;

    pub fn add_nft(&mut self, item: StakedNFT) {
        self.items[self.item_count as usize] = item;
        self.item_count += 1;
    }
    
    pub fn remove_nft(&mut self, owner: Pubkey, item_id: u64, daily_drip_percentage: u64) -> Result<u64> {
        require!(self.owner.eq(&owner), ThogError::OnlyOwnerCanCall);
        let mut withdrawn: u8 = 0;
        let mut reward: u64 = 0;
        let num: u64 = self.item_count;
        for i in 0..num {
            let index = i as usize;
            if self.items[index].item_id.eq(&item_id) {
                
                reward = calculate_user_reward(self.items[index].lock_time, self.items[index].period, self.items[index].staked_amount, self.items[index].last_update_time, self.items[index].multiplier, daily_drip_percentage);

                self.items[index] = StakedNFT {
                    ..Default::default()
                };

                self.item_count -= 1;
                withdrawn = 1;
                break;
            }
        }
        require!(withdrawn == 1, ThogError::InvalidNFTAddress);
        let mut temp_nfts: [StakedNFT; NFT_NUMBER] = Default::default();
        let mut temp_index: u64 = 0;
        for i in 0..num {
            let index = i as usize;
            if self.items[index].lock_time != 0 {
                temp_nfts[temp_index as usize] = self.items[index].clone();
                temp_index += 1;
            }
        }
        self.items = temp_nfts;
        Ok(reward)
    }

    // pub fn claim_reward(&mut self, owner: Pubkey, lock_time: u64, now: u64, reward_per_second: u64, total_weight: u64) -> Result<u64> {
    //     require!(self.owner.eq(&owner), ThogError::OnlyOwnerCanCall);
    //     let mut reward: u64 = 0;
    //     for i in 0..self.item_count {
    //         let index = i as usize;
    //         if self.items[index].lock_time.eq(&lock_time) {
    //             reward = calculate_user_reward(self.items[index].staked_amount, self.items[index].last_update_time, self.items[index].multiplier, reward_per_second, total_weight) as u64;
    //             self.items[index].last_update_time = now;
    //         }
    //     }
    //     Ok(reward)
    // }

    // pub fn claim_reward_all(&mut self, now: u64, reward_per_second: u64, total_weight: u64) -> Result<u64> {
    //     let mut total_reward: u64 = 0;
    //     for i in 0..self.item_count {
    //         let index = i as usize;
    //         let mut _reward: u64 = 0;
    //         _reward = calculate_user_reward(self.items[index].staked_amount, self.items[index].last_update_time, self.items[index].multiplier, reward_per_second, total_weight) as u64;
    //         total_reward += _reward;
    //         self.items[index].last_update_time = now;
    //     }
    //     Ok(total_reward)
    // }
}
