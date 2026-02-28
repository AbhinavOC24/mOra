use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        self,
        Mint,
        TokenAccount,
        TokenInterface,
        TransferChecked,
    },
};

declare_id!("E7TZEgFboKppM4V8yix6UEQrBh2WL2rDqbbn2JYxPnat");

#[program]
pub mod oo_core {
    use super::*;

    /* ---------------- Initialize Global ---------------- */

    pub fn initialize_global_config(
        ctx: Context<InitializeGlobalConfig>,
        min_proposer_bond: u64,
        arbiter_reward_bps:u16,
        burn_vault:Pubkey,
        min_arbiter_voting_power:u128,
        vla_commit_window:i64,
        vla_enabled:bool,
    ) -> Result<()> {
        let global = &mut ctx.accounts.global_config;
        let admin_key = ctx.accounts.admin.key();

        global.admin = admin_key;
        global.min_proposer_bond = min_proposer_bond;
        global. arbiter_reward_bps= arbiter_reward_bps;
        global.burn_vault= burn_vault;
        global.min_arbiter_voting_power= min_arbiter_voting_power;
        global.vla_commit_window= vla_commit_window;
        global.vla_enabled= vla_enabled;

        msg!("Global config initialized by {}", admin_key);
        Ok(())
    }

    /* ---------------- Request Assertion ---------------- */

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
        require!(question.len() <= 500, Error::QuestionTooLong);
        require!(requester_bond_amount > 0, Error::InvalidBondAmount);

        require!(
            (3600..=604800).contains(&dispute_period),
            Error::InvalidDisputePeriod
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

        // lifecycle fields
        a.proposer = None;
        a.proposer_bond = None;


        a.disputer = None;
        a.disputer_bond = None;

        a.proposed_answer = None;
        a.final_answer = None;
        a.resolved_at = None;
        a.status = AssertionStatus::Requested;

        // Move (bond + reward) MORA from requester ATA -> assertion escrow
        let total_amount = requester_bond_amount
            .checked_add(reward_amount)
            .ok_or(Error::InvalidBondAmount)?;

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

    /* ---------------- Propose Assertion ---------------- */

    pub fn propose_assertion(
        ctx: Context<ProposeAssertion>,
        proposed_answer: String,
        proposer_bond_amount: u64,
    ) -> Result<()> {
        let a = &mut ctx.accounts.assertion_request;
        let g = &ctx.accounts.global_config;

        let now=Clock::get()?.unix_timestamp;

 
        require!(
            now <= a.requested_at + a.liveness_period,
            Error::ProposalTooLate
        );
        require!(
            ctx.accounts.proposer.key() != a.requester,
            Error::RequesterCannotPropose
        );


        require!(
        a.status == AssertionStatus::Requested,
        Error::InvalidAssertionState
        );

        require!(!proposed_answer.is_empty(), Error::EmptyAnswerProposed);
        require!(
            proposer_bond_amount >= g.min_proposer_bond,
            Error::InvalidBondAmount
        );

        match a.answer_type {
            AnswerType::YesNo => {
                let lower = proposed_answer.to_lowercase();
                require!(lower == "yes" || lower == "no", Error::AnswerTypeMismatch);
            }
            AnswerType::Number => {
                require!(proposed_answer.parse::<i64>().is_ok(), Error::AnswerTypeMismatch);
            }
            AnswerType::String => {
                require!(proposed_answer.len() <= 500, Error::AnswerTypeMismatch);
            }
        }


        let cpi_accounts=TransferChecked{
            mint:ctx.accounts.mora_mint.to_account_info(),
            from:ctx.accounts.proposer_mora_ata.to_account_info(),
            to:ctx.accounts.assertion_escrow.to_account_info(),
            authority:ctx.accounts.proposer.to_account_info(),
        };

        let cpi_ctx=CpiContext::new( ctx.accounts.token_program.to_account_info(),
        cpi_accounts);

        let decimals = ctx.accounts.mora_mint.decimals;

        token_interface::transfer_checked(cpi_ctx,proposer_bond_amount,decimals)?;


        a.proposer = Some(ctx.accounts.proposer.key());
        a.proposer_bond = Some(proposer_bond_amount);


        let parsed_answer=match a.answer_type {
            AnswerType::YesNo =>{
                let lowercase=proposed_answer.to_lowercase();
                AnswerValue::YesNo(lowercase=="yes")
            }
            AnswerType::Number =>{
                let n =proposed_answer.parse::<i64>()
                .map_err(|_| Error::AnswerTypeMismatch)?;
                AnswerValue::Number(n)
            }
            AnswerType::String => AnswerValue::String(proposed_answer),

        };
      
        
        a.proposed_answer = Some(parsed_answer);
        a.proposed_at=Some(now);
        a.status = AssertionStatus::Proposed;



        msg!("Proposer {:?} submitted an answer", a.proposer.unwrap());
        Ok(())
    }

    /* ---------------- Auto Resolve Assertion ---------------- */

    pub fn auto_resolve_assertion(
        ctx: Context<AutoResolveAssertion>,
        _assertion_id: u64,
    ) -> Result<()> {
        let req = &mut ctx.accounts.assertion_request;
        let now= Clock::get()?.unix_timestamp;


            require!(
     req.status == AssertionStatus::Proposed,
     Error::InvalidAssertionState
    );  

   

        require!(
            req.requested_at + req.liveness_period <= Clock::get()?.unix_timestamp,
            Error::RequestTooEarly
        );
            req.final_answer = req.proposed_answer.clone();

            let bump = ctx.bumps.assertion_request;
let seeds: [&[u8]; 3] = [
    b"assertion",
    &_assertion_id.to_le_bytes(),
    &[bump],
];


let signer: &[&[&[u8]]] = &[&seeds];

let decimals = ctx.accounts.mora_mint.decimals;

// payout = proposer bond + reward
let payout_amount = req.reward_amount
    .checked_add(req.proposer_bond.unwrap())
    .ok_or(Error::InvalidBondAmount)?;

let transfer_accounts = TransferChecked {
    mint: ctx.accounts.mora_mint.to_account_info(),
    from: ctx.accounts.assertion_escrow.to_account_info(),
    to: ctx.accounts.proposer_mora_ata.to_account_info(),
    authority: req.to_account_info(),
};

let transfer_ctx: CpiContext<'_, '_, '_, '_, TransferChecked<'_>> = CpiContext::new_with_signer(
    ctx.accounts.token_program.to_account_info(),
    transfer_accounts,
    signer,
);

token_interface::transfer_checked(
    transfer_ctx,
    payout_amount,
    decimals,
)?;


    let requester_refund=req.requester_bond_amount;

    let refund_accounts= TransferChecked{
        from: ctx.accounts.assertion_escrow.to_account_info(),
        to:ctx.accounts.requester_mora_ata.to_account_info(),
        mint:ctx.accounts.mora_mint.to_account_info(),
        authority: req.to_account_info(),
    };


    let refund_ctx= CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
         refund_accounts,
        signer);


     let res=   token_interface::transfer_checked(
            refund_ctx,
            requester_refund,
            decimals
        );
        
        if res.is_err() {
            msg!("Failed to refund requester");
        }
        

        let close_assertion_escorw= token_interface::CloseAccount{
            account: ctx.accounts.assertion_escrow.to_account_info(),
            destination: ctx.accounts.requester.to_account_info(),
            authority: req.to_account_info(),
        };
        
        let close_ctx= CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_assertion_escorw,
            signer);

        token_interface::close_account(close_ctx)?;
        

        req.resolved_at = Some(Clock::get()?.unix_timestamp);

        req.status =AssertionStatus::Resolved;

        msg!(
            "Assertion {} auto-resolved with answer",
            req.assertion_id
        );
    


        Ok(())
    }



    /* ---------------- Dispute Assertion ---------------- */

    pub fn dispute_assertion(
    ctx: Context<DisputeAssertion>,
    assertion_id:u64,
    dispute_bond: u64,
) -> Result<()> {
    let req = &mut ctx.accounts.assertion_request;
    let now = Clock::get()?.unix_timestamp;

       require!(
    req.status == AssertionStatus::Proposed,
    Error::InvalidAssertionState
    );

    require!(
        req.requested_at+req.liveness_period>=now, Error::TooLateToDispute
    );
    require!(dispute_bond > 0, Error::InvalidBondAmount);
    
    let dispute_bond_acc= TransferChecked{
        mint:ctx.accounts.mora_mint.to_account_info(),
        from: ctx.accounts.disputer_mora_ata.to_account_info(),
        to: ctx.accounts.assertion_escrow.to_account_info(),
        authority: ctx.accounts.disputer.to_account_info(),
        
    };

    let dispute_bond_ctx= CpiContext::new(ctx.accounts.token_program.to_account_info(), dispute_bond_acc);

    let decimals = ctx.accounts.mora_mint.decimals;

    token_interface::transfer_checked(dispute_bond_ctx, dispute_bond, decimals)?;

    req.disputer = Some(ctx.accounts.disputer.key());
    req.disputer_bond = Some(dispute_bond);
    req.status = AssertionStatus::Disputed;

    msg!("Assertion disputed by {}", ctx.accounts.disputer.key());
    Ok(())
}



    /* ---------------- VLA (Value Locked Arbit) ---------------- */
    
    // fn open_vla_round

    /* ---------------- Initialize Locking Config (veMORA) ---------------- */

    pub fn initialize_locking_config(
        ctx: Context<InitializeLockingConfig>,
        mora_mint: Pubkey,
        min_lock_duration:i64,
        max_lock_duration: i64,
    ) -> Result<()> {
        require!(max_lock_duration > 0, LockError::InvalidMaxLockDuration);

        let config = &mut ctx.accounts.locking_config;
        config.admin = ctx.accounts.admin.key();
        config.mora_mint = mora_mint;
        config.min_lock_duration=min_lock_duration;

        config.max_lock_duration = max_lock_duration;

        Ok(())
    }

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

        // Transfer MORA → escrow
        let cpi_accounts = TransferChecked {
            mint: ctx.accounts.mora_mint.to_account_info(),
            from: ctx.accounts.owner_mora_ata.to_account_info(),
            to: ctx.accounts.lock_escrow.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_ctx =
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

        // Init lock
        lock.owner = ctx.accounts.owner.key();
        lock.amount_locked = amount;
        lock.lock_start = now;
        lock.lock_end = lock_end;

        recalc_voting_power(lock, config.max_lock_duration, now)
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
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
            );
            token_interface::transfer_checked(cpi_ctx, additional_amount, decimals)?;

            lock.amount_locked = lock
                .amount_locked
                .checked_add(additional_amount)
                .ok_or(LockError::MathOverflow)?;
        }

        lock.lock_end = new_lock_end;

        recalc_voting_power(lock, config.max_lock_duration, now)
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

        // Transfer back to owner
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

            token_interface::transfer_checked(
                transfer_ctx,
                lock.amount_locked,
                decimals,
            )?;
        }

        // Close if empty
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

        // Wipe lock
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
}


fn recalc_voting_power(
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

    let slope = (lock.amount_locked as u128)
        .checked_div(max_duration)
        .ok_or(LockError::MathOverflow)?;
    let bias = slope.saturating_mul(remaining);

    lock.slope = slope;
    lock.bias = bias;
    lock.voting_power = bias;

    Ok(())
}

/* ---------------- Accounts ---------------- */

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
        associated_token::authority=requester,
        associated_token::token_program=token_program,
    )]
    pub requester_mora_ata: InterfaceAccount<'info, TokenAccount>,

    pub mora_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,

    pub auto_resolver: Signer<'info>,
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
        payer=requester,
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

    #[account(
        constraint = mora_mint.key() != Pubkey::default()
    )]
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

    #[account(
        mut,
        seeds = [b"global"],
        bump
    )]
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
        associated_token::mint=mora_mint,
        associated_token::authority=proposer,
        associated_token::token_program=token_program,
    )]
    pub proposer_mora_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program:Interface<'info,TokenInterface>,
    pub mora_mint:InterfaceAccount<'info,Mint>,
    pub associated_token_program:Program<'info,AssociatedToken>
    
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

    #[account(mut,
        seeds=[b"assertion_escrow", assertion_id.to_le_bytes().as_ref()],
        bump,
        token::mint =mora_mint,
        token::authority= assertion_request,
        token::token_program = token_program,
    )]

    pub assertion_escrow:InterfaceAccount<'info,TokenAccount>,


    #[account(
        mut,
        associated_token::mint = mora_mint,
        associated_token::authority= disputer,
        associated_token::token_program = token_program
    )]
    pub disputer_mora_ata: InterfaceAccount <'info,TokenAccount>,

    pub mora_mint:InterfaceAccount<'info,Mint>,
    pub token_program:Interface<'info,TokenInterface>,

    #[account(mut)]
    pub disputer: Signer<'info>,


    #[account(
        init,
        payer=disputer,
        space=8 + VlaRound::INIT_SPACE,
        seeds=[b"vla_round",assertion_id.to_le_bytes().as_ref()],
        bump

    )]
    pub vla_round:Account<'info,VlaRound>,

    pub system_program: Program<'info, System>,
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

#[derive(Accounts)]
pub struct CreateLock<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"locking_config"],
        bump
    )]
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

    #[account(
        constraint = mora_mint.key() == locking_config.mora_mint
            @ LockError::WrongMint
    )]
    pub mora_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IncreaseLock<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"locking_config"],
        bump
    )]
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

    #[account(
        constraint = mora_mint.key() == locking_config.mora_mint
            @ LockError::WrongMint
    )]
    pub mora_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefreshVotingPower<'info> {
    #[account(
        seeds = [b"locking_config"],
        bump
    )]
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

    #[account(
        seeds = [b"locking_config"],
        bump
    )]
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

    #[account(
        constraint = mora_mint.key() == locking_config.mora_mint
            @ LockError::WrongMint
    )]
    pub mora_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/* ---------------- Data ---------------- */

#[account]
#[derive(InitSpace)]
pub struct Global {
    pub admin: Pubkey,
    pub min_proposer_bond: u64,

    pub arbiter_reward_bps:u16,
    pub burn_vault: Pubkey,

    pub min_arbiter_voting_power: u128,
    pub vla_commit_window: i64,

    pub vla_enabled:    bool,
}

#[account]
#[derive(InitSpace)]
pub struct LockingConfig {
    pub admin: Pubkey,
    pub mora_mint: Pubkey,
    pub min_lock_duration: i64,
    pub max_lock_duration: i64,
}

#[account]
#[derive(InitSpace)]
pub struct LockPosition {
    pub owner: Pubkey,
    pub amount_locked: u64,
    pub lock_start: i64,
    pub lock_end: i64,
    pub slope: u128,
    pub bias: u128,
    pub voting_power: u128,
}

#[account]
#[derive(InitSpace)]
pub struct AssertionRequest {
    pub assertion_id: u64,
    pub requester: Pubkey,

    #[max_len(512)]
    pub question: String,

    pub requester_bond_amount: u64,
    pub reward_amount: u64,

    pub answer_type: AnswerType,

    pub liveness_period: i64,
    pub dispute_period: i64,

    pub status: AssertionStatus,

    pub requested_at: i64,
    pub resolved_at: Option<i64>,

    pub proposer: Option<Pubkey>,
    pub proposed_at:Option<i64>,
    pub proposer_bond: Option<u64>,

    pub disputer: Option<Pubkey>,
    pub disputer_bond: Option<u64>,

    #[max_len(512)]
    pub proposed_answer: Option<AnswerValue>,

    #[max_len(512)]
    pub final_answer: Option<AnswerValue>,

    #[max_len(512)]
    pub metadata: Option<String>,
}

#[account]
pub struct AssertionEscrow {}



#[derive(InitSpace)]
#[account]
pub struct VlaRound{
    pub assertion_id:u64,

    pub opened_at:i64,
    pub voting_ends_at:i64,

    pub status: VlaRoundStatus,

    pub total_power_disputer:u128,
    pub totoal_power_proposer:u128,

}

pub enum VlaSide{
    Proposer,
    Disputer,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum VlaRoundStatus {
    InVLA,
    Closed, 
 }



#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum AnswerType {
    YesNo,
    Number,
    String,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum AssertionStatus {
    Requested,
    Proposed,
    Disputed,
    Resolved,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Eq)]
pub enum AnswerValue {
    YesNo(bool),      // true = yes, false = no
    Number(i64),      // numeric answers


    String(#[max_len(512)] String),
     // string answers
}
/* ---------------- Errors ---------------- */

#[error_code]
pub enum Error {
    #[msg("Dispute window over")]
    TooLateToDispute,
    #[msg("Invalid assertion lifecycle state for this action")]
InvalidAssertionState,
    #[msg("Cannot resolve this request yet")]
    RequestTooEarly,
    #[msg("Proposal submitted after liveness window ended")]
ProposalTooLate,
    #[msg("Question too long")]
    QuestionTooLong,
    #[msg("Invalid bond amount")]
    InvalidBondAmount,
    
    #[msg("Dispute period out of range")]
    InvalidDisputePeriod,
    #[msg("This request already has a proposed answer")]
    AlreadyProposed,
    #[msg("Cannot propose an empty answer")]
    EmptyAnswerProposed,
    #[msg("Invalid answer for selected AnswerType")]
    AnswerTypeMismatch,
    #[msg("Cannot resolve without a proposer")]
    NoProposerToResolve,
    #[msg("Requester cannot act as proposer on their own assertion")]
RequesterCannotPropose,
#[msg("Cannot Resolve when the assertion is disputed")]
AssertionIsDisputed,
}

#[error_code]
pub enum LockError {
    #[msg("Invalid max lock duration")]
    InvalidMaxLockDuration,
    #[msg("Lock duration must be positive")]
    NonPositiveDuration,
    #[msg("Lock duration too short")]
    LockTooShort,
    #[msg("Cannot shorten lock")]
    CannotShortenLock,
    #[msg("Lock not expired")]
    LockNotExpired,
    #[msg("Lock amount is zero")]
    ZeroAmount,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Wrong Mint Account Provided")]
    WrongMint,
}
