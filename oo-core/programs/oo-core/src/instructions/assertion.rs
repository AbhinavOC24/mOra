use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use crate::state::*;
use crate::errors::*;

pub fn request_assertion(
    ctx: Context<RequestAssertion>,
    assertion_id: u64,
    question: String,
    answer_type: AnswerType,
    liveness_period: i64,
    dispute_period: i64,
    metadata: Option<String>,
    requester_bond_amount: u64,
    reward_amount: u64,
) -> Result<()> {
    require!(question.len() <= 500, MoraError::QuestionTooLong);
    require!(requester_bond_amount > 0, MoraError::InvalidBondAmount);

    require!(
        (3600..=604800).contains(&dispute_period),
        MoraError::InvalidDisputePeriod
    );

    let a = &mut ctx.accounts.assertion_request;

    a.assertion_id = assertion_id;
    a.requester = ctx.accounts.requester.key();
    a.requested_at = Clock::get()?.unix_timestamp;

    a.reward_amount = reward_amount;
    a.requester_bond_amount = requester_bond_amount;

    a.question = question;
    a.answer_type = answer_type;
    a.liveness_period = liveness_period;
    a.dispute_period = dispute_period;
    a.metadata = metadata;

    a.proposer = None;
    a.proposer_bond = None;
    a.disputer = None;
    a.disputer_bond = None;
    a.proposed_answer = None;
    a.final_answer = None;
    a.resolved_at = None;
    a.status = AssertionStatus::Requested;

    let total_amount = requester_bond_amount
        .checked_add(reward_amount)
        .ok_or(MoraError::InvalidBondAmount)?;

    let decimals = ctx.accounts.mora_mint.decimals;

    let cpi_accounts = TransferChecked {
        mint: ctx.accounts.mora_mint.to_account_info(),
        from: ctx.accounts.requester_mora_ata.to_account_info(),
        to: ctx.accounts.assertion_escrow.to_account_info(),
        authority: ctx.accounts.requester.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );

    token_interface::transfer_checked(cpi_ctx, total_amount, decimals)?;

    msg!("Assertion request created with ID: {}", assertion_id);
    Ok(())
}

pub fn propose_assertion(
    ctx: Context<ProposeAssertion>,
    proposed_answer: String,
    proposer_bond_amount: u64,
) -> Result<()> {
    let a = &mut ctx.accounts.assertion_request;
    let g = &ctx.accounts.global_config;
    let now = Clock::get()?.unix_timestamp;

    require!(
        now <= a.requested_at + a.liveness_period,
        MoraError::ProposalTooLate
    );
    require!(
        ctx.accounts.proposer.key() != a.requester,
        MoraError::RequesterCannotPropose
    );
    require!(
        a.status == AssertionStatus::Requested,
        MoraError::InvalidAssertionState
    );
    require!(!proposed_answer.is_empty(), MoraError::EmptyAnswerProposed);
    require!(
        proposer_bond_amount >= g.min_proposer_bond,
        MoraError::InvalidBondAmount
    );

    match a.answer_type {
        AnswerType::YesNo => {
            let lower = proposed_answer.to_lowercase();
            require!(lower == "yes" || lower == "no", MoraError::AnswerTypeMismatch);
        }
        AnswerType::Number => {
            require!(proposed_answer.parse::<i64>().is_ok(), MoraError::AnswerTypeMismatch);
        }
        AnswerType::String => {
            require!(proposed_answer.len() <= 500, MoraError::AnswerTypeMismatch);
        }
    }

    let decimals = ctx.accounts.mora_mint.decimals;
    let cpi_accounts = TransferChecked {
        mint: ctx.accounts.mora_mint.to_account_info(),
        from: ctx.accounts.proposer_mora_ata.to_account_info(),
        to: ctx.accounts.assertion_escrow.to_account_info(),
        authority: ctx.accounts.proposer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token_interface::transfer_checked(cpi_ctx, proposer_bond_amount, decimals)?;

    a.proposer = Some(ctx.accounts.proposer.key());
    a.proposer_bond = Some(proposer_bond_amount);

    let parsed_answer = match a.answer_type {
        AnswerType::YesNo => {
            let lowercase = proposed_answer.to_lowercase();
            AnswerValue::YesNo(lowercase == "yes")
        }
        AnswerType::Number => {
            let n = proposed_answer.parse::<i64>().map_err(|_| MoraError::AnswerTypeMismatch)?;
            AnswerValue::Number(n)
        }
        AnswerType::String => AnswerValue::String(proposed_answer),
    };

    a.proposed_answer = Some(parsed_answer);
    a.proposed_at = Some(now);
    a.status = AssertionStatus::Proposed;

    msg!("Proposer {:?} submitted an answer", a.proposer.unwrap());
    Ok(())
}

pub fn auto_resolve_assertion(
    ctx: Context<AutoResolveAssertion>,
    assertion_id: u64,
) -> Result<()> {
    let req = &mut ctx.accounts.assertion_request;
    let _now = Clock::get()?.unix_timestamp;

    require!(
        req.status == AssertionStatus::Proposed,
        MoraError::InvalidAssertionState
    );
    require!(
        req.requested_at + req.liveness_period <= Clock::get()?.unix_timestamp,
        MoraError::RequestTooEarly
    );
    
    req.final_answer = req.proposed_answer.clone();

    let bump = ctx.bumps.assertion_request;
    let seeds: [&[u8]; 3] = [
        b"assertion",
        &assertion_id.to_le_bytes(),
        &[bump],
    ];
    let signer: &[&[&[u8]]] = &[&seeds];
    let decimals = ctx.accounts.mora_mint.decimals;

    let payout_amount = req.reward_amount
        .checked_add(req.proposer_bond.unwrap())
        .ok_or(MoraError::InvalidBondAmount)?;

    let transfer_accounts = TransferChecked {
        mint: ctx.accounts.mora_mint.to_account_info(),
        from: ctx.accounts.assertion_escrow.to_account_info(),
        to: ctx.accounts.proposer_mora_ata.to_account_info(),
        authority: req.to_account_info(),
    };
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
        signer,
    );
    token_interface::transfer_checked(transfer_ctx, payout_amount, decimals)?;

    let requester_refund = req.requester_bond_amount;
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
    let _ = token_interface::transfer_checked(refund_ctx, requester_refund, decimals);

    let close_assertion_escrow = token_interface::CloseAccount {
        account: ctx.accounts.assertion_escrow.to_account_info(),
        destination: ctx.accounts.requester.to_account_info(),
        authority: req.to_account_info(),
    };
    let close_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        close_assertion_escrow,
        signer,
    );
    token_interface::close_account(close_ctx)?;

    req.resolved_at = Some(Clock::get()?.unix_timestamp);
    req.status = AssertionStatus::Resolved;

    msg!("Assertion {} auto-resolved", req.assertion_id);
    Ok(())
}

pub fn dispute_assertion(
    ctx: Context<DisputeAssertion>,
    assertion_id: u64,
    dispute_bond: u64,
) -> Result<()> {
    let req = &mut ctx.accounts.assertion_request;
    let now = Clock::get()?.unix_timestamp;

    require!(
        req.status == AssertionStatus::Proposed,
        MoraError::InvalidAssertionState
    );
    require!(
        req.requested_at + req.liveness_period >= now,
        MoraError::TooLateToDispute
    );
    require!(dispute_bond > 0, MoraError::InvalidBondAmount);

    let decimals = ctx.accounts.mora_mint.decimals;
    let dispute_bond_acc = TransferChecked {
        mint: ctx.accounts.mora_mint.to_account_info(),
        from: ctx.accounts.disputer_mora_ata.to_account_info(),
        to: ctx.accounts.assertion_escrow.to_account_info(),
        authority: ctx.accounts.disputer.to_account_info(),
    };
    let dispute_bond_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), dispute_bond_acc);
    token_interface::transfer_checked(dispute_bond_ctx, dispute_bond, decimals)?;

    req.disputer = Some(ctx.accounts.disputer.key());
    req.disputer_bond = Some(dispute_bond);
    req.status = AssertionStatus::Disputed;

    let vla = &mut ctx.accounts.vla_round;
    let global = &ctx.accounts.global_config;

    vla.assertion_id = assertion_id;
    vla.opened_at = now;
    vla.commit_ends_at = now + global.vla_commit_window;
    vla.reveal_ends_at = now + global.vla_commit_window * 2;
    vla.status = VlaRoundStatus::CommitPhase;
    vla.total_power_disputer = 0;
    vla.total_power_proposer = 0;

    msg!("Assertion disputed by {}. VLA Round opened.", ctx.accounts.disputer.key());
    Ok(())
}

#[derive(Accounts)]
#[instruction(assertion_id: u64)]
pub struct RequestAssertion<'info> {
    #[account(mut)]
    pub requester: Signer<'info>,
    #[account(
        init,
        payer = requester,
        space = 8 + AssertionRequest::INIT_SPACE,
        seeds = [b"assertion", assertion_id.to_le_bytes().as_ref()],
        bump
    )]
    pub assertion_request: Account<'info, AssertionRequest>,
    #[account(
        init,
        payer = requester,
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
        associated_token::authority = requester,
        associated_token::token_program = token_program,
    )]
    pub requester_mora_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(constraint = mora_mint.key() != Pubkey::default())]
    pub mora_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(assertion_id: u64)]
pub struct ProposeAssertion<'info> {
    #[account(
        mut,
        seeds = [b"assertion", assertion_id.to_le_bytes().as_ref()],
        bump
    )]
    pub assertion_request: Account<'info, AssertionRequest>,
    #[account(seeds = [b"global"], bump)]
    pub global_config: Account<'info, Global>,
    #[account(mut)]
    pub proposer: Signer<'info>,
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
        associated_token::authority = proposer,
        associated_token::token_program = token_program,
    )]
    pub proposer_mora_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub mora_mint: InterfaceAccount<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(assertion_id: u64)]
pub struct AutoResolveAssertion<'info> {
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
        associated_token::authority = requester,
        associated_token::token_program = token_program,
    )]
    pub requester_mora_ata: InterfaceAccount<'info, TokenAccount>,
    pub mora_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub auto_resolver: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(assertion_id: u64)]
pub struct DisputeAssertion<'info> {
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
        associated_token::authority = disputer,
        associated_token::token_program = token_program
    )]
    pub disputer_mora_ata: InterfaceAccount<'info, TokenAccount>,
    pub mora_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    #[account(mut)]
    pub disputer: Signer<'info>,
    #[account(
        init,
        payer = disputer,
        space = 8 + VlaRound::INIT_SPACE,
        seeds = [b"vla_round", assertion_id.to_le_bytes().as_ref()],
        bump
    )]
    pub vla_round: Account<'info, VlaRound>,
    #[account(seeds = [b"global"], bump)]
    pub global_config: Account<'info, Global>,
    pub system_program: Program<'info, System>,
}
