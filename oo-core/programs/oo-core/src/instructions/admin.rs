use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

pub fn initialize_global_config(
    ctx: Context<InitializeGlobalConfig>,
    min_proposer_bond: u64,
    arbiter_reward_bps: u16,
    burn_vault: Pubkey,
    min_arbiter_voting_power: u128,
    vla_commit_window: i64,
    vla_enabled: bool,
) -> Result<()> {
    let global = &mut ctx.accounts.global_config;
    let admin_key = ctx.accounts.admin.key();

    global.admin = admin_key;
    global.min_proposer_bond = min_proposer_bond;
    global.arbiter_reward_bps = arbiter_reward_bps;
    global.burn_vault = burn_vault;
    global.min_arbiter_voting_power = min_arbiter_voting_power;
    global.vla_commit_window = vla_commit_window;
    global.vla_enabled = vla_enabled;

    msg!("Global config initialized by {}", admin_key);
    Ok(())
}

pub fn initialize_locking_config(
    ctx: Context<InitializeLockingConfig>,
    mora_mint: Pubkey,
    min_lock_duration: i64,
    max_lock_duration: i64,
) -> Result<()> {
    require!(max_lock_duration > 0, LockError::InvalidMaxLockDuration);

    let config = &mut ctx.accounts.locking_config;
    config.admin = ctx.accounts.admin.key();
    config.mora_mint = mora_mint;
    config.min_lock_duration = min_lock_duration;
    config.max_lock_duration = max_lock_duration;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeGlobalConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Global::INIT_SPACE,
        seeds = [b"global"],
        bump
    )]
    pub global_config: Account<'info, Global>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeLockingConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + LockingConfig::INIT_SPACE,
        seeds = [b"locking_config"],
        bump
    )]
    pub locking_config: Account<'info, LockingConfig>,

    pub system_program: Program<'info, System>,
}
