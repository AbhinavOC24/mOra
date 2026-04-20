use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use crate::state::*;
use crate::errors::*;

pub fn create_lock(
    ctx: Context<CreateLock>,
    amount: u64,
    lock_duration: i64,
) -> Result<()> {
    let config = &ctx.accounts.locking_config;
    let lock = &mut ctx.accounts.lock_position;
    let now = Clock::get()?.unix_timestamp;

    require!(amount > 0, LockError::ZeroAmount);
    require!(lock_duration > 0, LockError::NonPositiveDuration);
    require!(
        lock_duration >= config.min_lock_duration,
        LockError::LockTooShort
    );
    require!(
        lock_duration <= config.max_lock_duration,
        LockError::InvalidMaxLockDuration
    );

    let lock_end = now + lock_duration;
    let decimals = ctx.accounts.mora_mint.decimals;

    let cpi_accounts = TransferChecked {
        mint: ctx.accounts.mora_mint.to_account_info(),
        from: ctx.accounts.owner_mora_ata.to_account_info(),
        to: ctx.accounts.lock_escrow.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    lock.owner = ctx.accounts.owner.key();
    lock.amount_locked = amount;
    lock.lock_start = now;
    lock.lock_end = lock_end;

    let config_mut = &mut ctx.accounts.locking_config;
    recalc_voting_power(lock, config_mut.max_lock_duration, now)?;
    
    config_mut.total_slope = config_mut.total_slope.checked_add(lock.slope).unwrap();
    config_mut.total_bias = config_mut.total_bias.checked_add(lock.bias).unwrap();
    config_mut.last_update_ts = now;

    Ok(())
}

pub fn increase_lock(
    ctx: Context<IncreaseLock>,
    additional_amount: u64,
    lock_duration: i64,
) -> Result<()> {
    let config = &ctx.accounts.locking_config;
    let lock = &mut ctx.accounts.lock_position;
    let now = Clock::get()?.unix_timestamp;

    require!(lock_duration > 0, LockError::NonPositiveDuration);
    require!(
        lock_duration >= config.min_lock_duration,
        LockError::LockTooShort
    );
    require!(
        lock_duration <= config.max_lock_duration,
        LockError::InvalidMaxLockDuration
    );

    let new_lock_end = now + lock_duration;
    require!(new_lock_end >= lock.lock_end, LockError::CannotShortenLock);

    let decimals = ctx.accounts.mora_mint.decimals;

    if additional_amount > 0 {
        let cpi_accounts = TransferChecked {
            mint: ctx.accounts.mora_mint.to_account_info(),
            from: ctx.accounts.owner_mora_ata.to_account_info(),
            to: ctx.accounts.lock_escrow.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token_interface::transfer_checked(cpi_ctx, additional_amount, decimals)?;

        lock.amount_locked = lock
            .amount_locked
            .checked_add(additional_amount)
            .ok_or(LockError::MathOverflow)?;
    }

    lock.lock_end = new_lock_end;
    
    let config_mut = &mut ctx.accounts.locking_config;
    let old_slope = lock.slope;
    let old_bias = lock.bias;

    recalc_voting_power(lock, config_mut.max_lock_duration, now)?;

    config_mut.total_slope = config_mut.total_slope.checked_sub(old_slope).unwrap().checked_add(lock.slope).unwrap();
    config_mut.total_bias = config_mut.total_bias.checked_sub(old_bias).unwrap().checked_add(lock.bias).unwrap();
    config_mut.last_update_ts = now;

    Ok(())
}

pub fn unlock(ctx: Context<Unlock>) -> Result<()> {
    let lock = &mut ctx.accounts.lock_position;
    let now = Clock::get()?.unix_timestamp;

    require!(now >= lock.lock_end, LockError::LockNotExpired);

    let bump = ctx.bumps.lock_position;
    let seeds: [&[u8]; 3] = [b"lock_position", lock.owner.as_ref(), &[bump]];
    let signer: &[&[u8]] = &seeds;
    let signer_seeds: &[&[&[u8]]] = &[signer];

    let decimals = ctx.accounts.mora_mint.decimals;

    if lock.amount_locked > 0 {
        let transfer_accounts = TransferChecked {
            mint: ctx.accounts.mora_mint.to_account_info(),
            from: ctx.accounts.lock_escrow.to_account_info(),
            to: ctx.accounts.owner_mora_ata.to_account_info(),
            authority: lock.to_account_info(),
        };
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );
        token_interface::transfer_checked(transfer_ctx, lock.amount_locked, decimals)?;
    }

    if ctx.accounts.lock_escrow.amount == 0 {
        let close_accounts = token_interface::CloseAccount {
            account: ctx.accounts.lock_escrow.to_account_info(),
            destination: ctx.accounts.owner.to_account_info(),
            authority: lock.to_account_info(),
        };
        let close_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_accounts,
            signer_seeds,
        );
        token_interface::close_account(close_ctx)?;
    }

    let config_mut = &mut ctx.accounts.locking_config;
    config_mut.total_slope = config_mut.total_slope.checked_sub(lock.slope).unwrap();
    config_mut.total_bias = config_mut.total_bias.checked_sub(lock.bias).unwrap();
    config_mut.last_update_ts = now;

    lock.amount_locked = 0;
    lock.lock_start = 0;
    lock.lock_end = now;
    lock.slope = 0;
    lock.bias = 0;
    lock.voting_power = 0;

    Ok(())
}

pub fn refresh_voting_power(ctx: Context<RefreshVotingPower>) -> Result<()> {
    let config = &ctx.accounts.locking_config;
    let now = Clock::get()?.unix_timestamp;
    recalc_voting_power(&mut ctx.accounts.lock_position, config.max_lock_duration, now)
}

pub fn recalc_voting_power(
    lock: &mut LockPosition,
    max_lock_duration: i64,
    now: i64,
) -> Result<()> {
    require!(max_lock_duration > 0, LockError::InvalidMaxLockDuration);

    if lock.lock_end <= now || lock.amount_locked == 0 {
        lock.slope = 0;
        lock.bias = 0;
        lock.voting_power = 0;
        return Ok(());
    }

    let max_duration = max_lock_duration as u128;
    let remaining = (lock.lock_end - now) as u128;

    // Use a scale factor of 10^12 for slope precision
    let scale: u128 = 1_000_000_000_000;
    
    let slope = (lock.amount_locked as u128)
        .checked_mul(scale)
        .ok_or(LockError::MathOverflow)?
        .checked_div(max_duration)
        .ok_or(LockError::MathOverflow)?;
        
    let bias = slope.saturating_mul(remaining) / scale;

    lock.slope = slope;
    lock.bias = bias;
    lock.voting_power = bias;

    Ok(())
}

#[derive(Accounts)]
pub struct CreateLock<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [b"locking_config"], bump)]
    pub locking_config: Account<'info, LockingConfig>,
    #[account(
        init,
        payer = owner,
        space = 8 + LockPosition::INIT_SPACE,
        seeds = [b"lock_position", owner.key().as_ref()],
        bump
    )]
    pub lock_position: Account<'info, LockPosition>,
    #[account(
        init,
        payer = owner,
        seeds = [b"lock_escrow", owner.key().as_ref()],
        bump,
        token::mint = mora_mint,
        token::authority = lock_position,
        token::token_program = token_program,
    )]
    pub lock_escrow: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mora_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program,
    )]
    pub owner_mora_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(constraint = mora_mint.key() == locking_config.mora_mint @ LockError::WrongMint)]
    pub mora_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IncreaseLock<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [b"locking_config"], bump)]
    pub locking_config: Account<'info, LockingConfig>,
    #[account(
        mut,
        seeds = [b"lock_position", owner.key().as_ref()],
        bump
    )]
    pub lock_position: Account<'info, LockPosition>,
    #[account(
        mut,
        seeds = [b"lock_escrow", owner.key().as_ref()],
        bump,
        token::mint = mora_mint,
        token::authority = lock_position,
        token::token_program = token_program,
    )]
    pub lock_escrow: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mora_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program,
    )]
    pub owner_mora_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(constraint = mora_mint.key() == locking_config.mora_mint @ LockError::WrongMint)]
    pub mora_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefreshVotingPower<'info> {
    #[account(seeds = [b"locking_config"], bump)]
    pub locking_config: Account<'info, LockingConfig>,
    #[account(
        mut,
        seeds = [b"lock_position", lock_position.owner.as_ref()],
        bump
    )]
    pub lock_position: Account<'info, LockPosition>,
}

#[derive(Accounts)]
pub struct Unlock<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [b"locking_config"], bump)]
    pub locking_config: Account<'info, LockingConfig>,
    #[account(
        mut,
        seeds = [b"lock_position", owner.key().as_ref()],
        bump
    )]
    pub lock_position: Account<'info, LockPosition>,
    #[account(
        mut,
        seeds = [b"lock_escrow", owner.key().as_ref()],
        bump,
        token::mint = mora_mint,
        token::authority = lock_position,
        token::token_program = token_program,
    )]
    pub lock_escrow: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mora_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program,
    )]
    pub owner_mora_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(constraint = mora_mint.key() == locking_config.mora_mint @ LockError::WrongMint)]
    pub mora_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
