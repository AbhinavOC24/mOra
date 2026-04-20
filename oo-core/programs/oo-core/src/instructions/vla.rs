use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use crate::state::*;
use crate::errors::*;
use crate::instructions::locking::recalc_voting_power;

pub fn commit_vote(
    ctx: Context<CommitVote>,
    assertion_id: u64,
    commit_hash: [u8; 32],
) -> Result<()> {
    let vla = &ctx.accounts.vla_round;
    let lock = &mut ctx.accounts.lock_position;
    let config = &ctx.accounts.locking_config;
    let now = Clock::get()?.unix_timestamp;

    require!(vla.status == VlaRoundStatus::CommitPhase, VlaError::NotCommitPhase);
    require!(now <= vla.commit_ends_at, VlaError::NotCommitPhase);
    
    recalc_voting_power(lock, config.max_lock_duration, now)?;
    
    require!(
        lock.voting_power >= ctx.accounts.global_config.min_arbiter_voting_power, 
        MoraError::InsufficientVotingPower
    );

    let vote = &mut ctx.accounts.vote_record;
    vote.voter = ctx.accounts.voter.key();
    vote.assertion_id = assertion_id;
    vote.commit_hash = commit_hash;
    vote.revealed = false;
    vote.side = None;
    vote.voting_power = lock.voting_power;
    vote.reward_claimed = false;

    msg!("Voter {} committed to assertion {}", vote.voter, assertion_id);
    Ok(())
}

pub fn reveal_vote(
    ctx: Context<RevealVote>,
    _assertion_id: u64,
    side: VlaSide,
    salt: u64,
) -> Result<()> {
    let vla = &mut ctx.accounts.vla_round;
    let vote = &mut ctx.accounts.vote_record;
    let now = Clock::get()?.unix_timestamp;

    if vla.status == VlaRoundStatus::CommitPhase && now > vla.commit_ends_at {
        vla.status = VlaRoundStatus::RevealPhase;
    }

    require!(vla.status == VlaRoundStatus::RevealPhase, VlaError::NotRevealPhase);
    require!(now <= vla.reveal_ends_at, VlaError::NotRevealPhase);
    require!(!vote.revealed, VlaError::AlreadyRevealed);

    let mut data = Vec::new();
    data.push(match side { VlaSide::Proposer => 0, VlaSide::Disputer => 1 });
    data.extend_from_slice(&salt.to_le_bytes());
    let hash = anchor_lang::solana_program::hash::hash(&data).to_bytes();
    
    require!(hash == vote.commit_hash, MoraError::CommitHashMismatch);

    vote.revealed = true;
    vote.side = Some(side);
    
    match side {
        VlaSide::Proposer => {
            vla.total_power_proposer = vla.total_power_proposer.checked_add(vote.voting_power).unwrap();
        }
        VlaSide::Disputer => {
            vla.total_power_disputer = vla.total_power_disputer.checked_add(vote.voting_power).unwrap();
        }
    }

    msg!("Voter {} revealed side: {:?}", vote.voter, side);
    Ok(())
}

pub fn finalize_vla_round(
    ctx: Context<FinalizeVlaRound>,
    assertion_id: u64,
) -> Result<()> {
    let vla = &mut ctx.accounts.vla_round;
    let req = &mut ctx.accounts.assertion_request;
    let global = &ctx.accounts.global_config;
    let now = Clock::get()?.unix_timestamp;

    require!(vla.status != VlaRoundStatus::Finalized, VlaError::VlaAlreadyClosed);
    require!(now > vla.reveal_ends_at, VlaError::VlaNotClosed);

    let total_voted = vla.total_power_proposer.checked_add(vla.total_power_disputer).unwrap();
    require!(total_voted >= global.quorum, MoraError::InvalidVlaPhase); // Or some other Quorum error

    let proposer_won = vla.total_power_proposer >= vla.total_power_disputer;
    let winning_side = if proposer_won { VlaSide::Proposer } else { VlaSide::Disputer };
    vla.winning_side = Some(winning_side);

    if proposer_won {
        req.final_answer = req.proposed_answer.clone();
        msg!("VLA result: Proposer Won");
    } else {
        req.final_answer = None; 
        msg!("VLA result: Disputer Won");
    }

    vla.status = VlaRoundStatus::Finalized;
    req.status = AssertionStatus::Resolved;
    req.resolved_at = Some(now);

    // Distribution logic setup
    let bump = ctx.bumps.assertion_request;
    let seeds: [&[u8]; 3] = [
        b"assertion",
        &assertion_id.to_le_bytes(),
        &[bump],
    ];
    let signer: &[&[&[u8]]] = &[&seeds];
    let decimals = ctx.accounts.mora_mint.decimals;

    // Refund Requester bond
    let refund_amount = req.requester_bond_amount;
    let refund_accounts = TransferChecked {
        from: ctx.accounts.assertion_escrow.to_account_info(),
        to: ctx.accounts.requester_mora_ata.to_account_info(),
        mint: ctx.accounts.mora_mint.to_account_info(),
        authority: req.to_account_info(),
    };
    let refund_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        refund_accounts,
        signer,
    );
    token_interface::transfer_checked(refund_ctx, refund_amount, decimals)?;

    // Give loser bond to winner
    let loser_bond = if proposer_won { req.disputer_bond.unwrap() } else { req.proposer_bond.unwrap() };
    let winner_ata = if proposer_won { ctx.accounts.proposer_mora_ata.to_account_info() } else { ctx.accounts.disputer_mora_ata.to_account_info() };
    
    let winner_payout = (if proposer_won { req.proposer_bond.unwrap() } else { req.disputer_bond.unwrap() })
        .checked_add(loser_bond).unwrap();

    let winner_accounts = TransferChecked {
        from: ctx.accounts.assertion_escrow.to_account_info(),
        to: winner_ata,
        mint: ctx.accounts.mora_mint.to_account_info(),
        authority: req.to_account_info(),
    };
    let winner_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        winner_accounts,
        signer,
    );
    token_interface::transfer_checked(winner_ctx, winner_payout, decimals)?;

    // Reward R stays in escrow for arbiters to claim pro-rata
    msg!("Requester bond refunded. Winner bond doubled. Arbiter rewards locked for claim.");
    
    Ok(())
}

pub fn claim_arbiter_reward(
    ctx: Context<ClaimArbiterReward>,
    assertion_id: u64,
) -> Result<()> {
    let vla = &ctx.accounts.vla_round;
    let vote = &mut ctx.accounts.vote_record;
    let req = &ctx.accounts.assertion_request;
    
    require!(vla.status == VlaRoundStatus::Finalized, VlaError::VlaNotClosed);
    require!(!vote.reward_claimed, VlaError::AlreadyRevealed);
    require!(vote.revealed, MoraError::InvalidVlaPhase);
    require!(vote.side == vla.winning_side, MoraError::InvalidVlaPhase);

    let winning_power = if vla.winning_side == Some(VlaSide::Proposer) { vla.total_power_proposer } else { vla.total_power_disputer };
    
    // Reward = R * (vote.vp / winning_power)
    let reward_amount = (req.reward_amount as u128)
        .checked_mul(vote.voting_power)
        .unwrap()
        .checked_div(winning_power)
        .unwrap() as u64;

    if reward_amount > 0 {
        let bump = ctx.bumps.assertion_request;
        let seeds: [&[u8]; 3] = [
            b"assertion",
            &assertion_id.to_le_bytes(),
            &[bump],
        ];
        let signer: &[&[&[u8]]] = &[&seeds];
        let decimals = ctx.accounts.mora_mint.decimals;

        let transfer_accounts = TransferChecked {
            from: ctx.accounts.assertion_escrow.to_account_info(),
            to: ctx.accounts.voter_mora_ata.to_account_info(),
            mint: ctx.accounts.mora_mint.to_account_info(),
            authority: req.to_account_info(),
        };
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer,
        );
        token_interface::transfer_checked(transfer_ctx, reward_amount, decimals)?;
    }

    vote.reward_claimed = true;
    msg!("Reward claimed: {}", reward_amount);
    Ok(())
}

#[derive(Accounts)]
#[instruction(assertion_id: u64)]
pub struct CommitVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(
        init,
        payer = voter,
        space = 8 + VoteRecord::INIT_SPACE,
        seeds = [b"vote_record", assertion_id.to_le_bytes().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    #[account(
        mut,
        seeds = [b"vla_round", assertion_id.to_le_bytes().as_ref()],
        bump
    )]
    pub vla_round: Account<'info, VlaRound>,
    #[account(
        mut,
        seeds = [b"lock_position", voter.key().as_ref()],
        bump
    )]
    pub lock_position: Account<'info, LockPosition>,
    #[account(seeds = [b"locking_config"], bump)]
    pub locking_config: Account<'info, LockingConfig>,
    #[account(seeds = [b"global"], bump)]
    pub global_config: Account<'info, Global>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(assertion_id: u64)]
pub struct RevealVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vote_record", assertion_id.to_le_bytes().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    #[account(
        mut,
        seeds = [b"vla_round", assertion_id.to_le_bytes().as_ref()],
        bump
    )]
    pub vla_round: Account<'info, VlaRound>,
}

#[derive(Accounts)]
#[instruction(assertion_id: u64)]
pub struct FinalizeVlaRound<'info> {
    #[account(
        mut,
        seeds = [b"assertion", assertion_id.to_le_bytes().as_ref()],
        bump
    )]
    pub assertion_request: Account<'info, AssertionRequest>,

    #[account(
        mut,
        seeds = [b"assertion_escrow", assertion_id.to_le_bytes().as_ref()],
        bump,
        token::mint = mora_mint,
        token::authority = assertion_request,
        token::token_program = token_program,
    )]
    pub assertion_escrow: InterfaceAccount<'info, TokenAccount>,

    pub proposer: SystemAccount<'info>,
    pub disputer: SystemAccount<'info>,
    pub requester: SystemAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mora_mint,
        associated_token::authority = proposer,
        associated_token::token_program = token_program,
    )]
    pub proposer_mora_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mora_mint,
        associated_token::authority = disputer,
        associated_token::token_program = token_program,
    )]
    pub disputer_mora_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mora_mint,
        associated_token::authority = requester,
        associated_token::token_program = token_program,
    )]
    pub requester_mora_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vla_round", assertion_id.to_le_bytes().as_ref()],
        bump
    )]
    pub vla_round: Account<'info, VlaRound>,

    pub mora_mint: InterfaceAccount<'info, Mint>,
    #[account(seeds = [b"global"], bump)]
    pub global_config: Account<'info, Global>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
#[instruction(assertion_id: u64)]
pub struct ClaimArbiterReward<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        mut,
        seeds = [b"assertion", assertion_id.to_le_bytes().as_ref()],
        bump
    )]
    pub assertion_request: Account<'info, AssertionRequest>,

    #[account(
        mut,
        seeds = [b"assertion_escrow", assertion_id.to_le_bytes().as_ref()],
        bump,
        token::mint = mora_mint,
        token::authority = assertion_request,
        token::token_program = token_program,
    )]
    pub assertion_escrow: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mora_mint,
        associated_token::authority = voter,
        associated_token::token_program = token_program,
    )]
    pub voter_mora_ata: InterfaceAccount<'info, TokenAccount>,

    pub vla_round: Account<'info, VlaRound>,
    pub vote_record: Account<'info, VoteRecord>,
    pub mora_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}
