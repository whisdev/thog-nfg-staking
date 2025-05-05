use anchor_lang::prelude::*;

#[error_code]
pub enum ThogError {
    #[msg("You are not authorized.")]
    Unautherized,
    #[msg("You are not super admin.")]
    InvalidAdmin,
    #[msg("Invalid item count.")]
    InvalidItemCount,
    #[msg("You can't unlock.")]
    LockPeriodNotOver,
    #[msg("You can withdraw after 24 hours.")]
    WithdrawNotOver,
    #[msg("You can withdraw after 1 day when you unlocked nft.")]
    WithdrawAfter,
    #[msg("You have already unlocked.")]
    AlreadyUnlocked,
    #[msg("Insufficient staked token amount.")]
    InsufficientStakedAmount,
    #[msg("Only owner can call this func.")]
    OnlyOwnerCanCall,
    #[msg("You can't unlock in LockTime")]
    BeforeLockTime,
    #[msg("No Matching NFT to withdraw")]
    InvalidNFTAddress,
    #[msg("Collection is invalid")]
    InvalidCollection,
    #[msg("Can not parse creators in metadata")]
    MetadataCreatorParseError,
    #[msg("time overflow")]
    TimeOverflow,
    #[msg("multiplication overflow")]
    MultiplicationOverflow,
    #[msg("Insufficient claim amount.")]
    InsufficientClaimAmount,
    #[msg("Insufficient reward vault amount.")]
    InsufficientRewardVaultBalance,
}
