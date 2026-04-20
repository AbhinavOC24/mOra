use anchor_lang::prelude::*;

pub mod errors;
pub mod state;
pub mod instructions;

use instructions::*;
use state::*;

declare_id!("E7TZEgFboKppM4V8yix6UEQrBh2WL2rDqbbn2JYxPnat");

#[program]
pub mod oo_core {
    use super::*;

    /* ---------------- Admin ---------------- */

    pub fn initialize_global_config(
        ctx: Context<InitializeGlobalConfig>,
        min_proposer_bond: u64,
        arbiter_reward_bps: u16,
        burn_vault: Pubkey,
        min_arbiter_voting_power: u128,
        vla_commit_window: i64,
        vla_enabled: bool,
    ) -> Result<()> {
        instructions::admin::initialize_global_config(
            ctx,
            min_proposer_bond,
            arbiter_reward_bps,
            burn_vault,
            min_arbiter_voting_power,
            vla_commit_window,
            vla_enabled,
        )
    }

    pub fn initialize_locking_config(
        ctx: Context<InitializeLockingConfig>,
        mora_mint: Pubkey,
        min_lock_duration: i64,
        max_lock_duration: i64,
    ) -> Result<()> {
        instructions::admin::initialize_locking_config(
            ctx,
            mora_mint,
            min_lock_duration,
            max_lock_duration,
        )
    }

    /* ---------------- Assertion ---------------- */

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
        instructions::assertion::request_assertion(
            ctx,
            assertion_id,
            question,
            answer_type,
            liveness_period,
            dispute_period,
            metadata,
            requester_bond_amount,
            reward_amount,
        )
    }

    pub fn propose_assertion(
        ctx: Context<ProposeAssertion>,
        proposed_answer: String,
        proposer_bond_amount: u64,
    ) -> Result<()> {
        instructions::assertion::propose_assertion(ctx, proposed_answer, proposer_bond_amount)
    }

    pub fn auto_resolve_assertion(
        ctx: Context<AutoResolveAssertion>,
        assertion_id: u64,
    ) -> Result<()> {
        instructions::assertion::auto_resolve_assertion(ctx, assertion_id)
    }

    pub fn dispute_assertion(
        ctx: Context<DisputeAssertion>,
        assertion_id: u64,
        dispute_bond: u64,
    ) -> Result<()> {
        instructions::assertion::dispute_assertion(ctx, assertion_id, dispute_bond)
    }

    /* ---------------- Locking (veMORA) ---------------- */

    pub fn create_lock(
        ctx: Context<CreateLock>,
        amount: u64,
        lock_duration: i64,
    ) -> Result<()> {
        instructions::locking::create_lock(ctx, amount, lock_duration)
    }

    pub fn increase_lock(
        ctx: Context<IncreaseLock>,
        additional_amount: u64,
        lock_duration: i64,
    ) -> Result<()> {
        instructions::locking::increase_lock(ctx, additional_amount, lock_duration)
    }

    pub fn unlock(ctx: Context<Unlock>) -> Result<()> {
        instructions::locking::unlock(ctx)
    }

    pub fn refresh_voting_power(ctx: Context<RefreshVotingPower>) -> Result<()> {
        instructions::locking::refresh_voting_power(ctx)
    }

    /* ---------------- VLA (Arbitration) ---------------- */

    pub fn commit_vote(
        ctx: Context<CommitVote>,
        assertion_id: u64,
        commit_hash: [u8; 32],
    ) -> Result<()> {
        instructions::vla::commit_vote(ctx, assertion_id, commit_hash)
    }

    pub fn reveal_vote(
        ctx: Context<RevealVote>,
        assertion_id: u64,
        side: VlaSide,
        salt: u64,
    ) -> Result<()> {
        instructions::vla::reveal_vote(ctx, assertion_id, side, salt)
    }

    pub fn finalize_vla_round(
        ctx: Context<FinalizeVlaRound>,
        assertion_id: u64,
    ) -> Result<()> {
        instructions::vla::finalize_vla_round(ctx, assertion_id)
    }

    pub fn claim_arbiter_reward(
        ctx: Context<ClaimArbiterReward>,
        assertion_id: u64,
    ) -> Result<()> {
        instructions::vla::claim_arbiter_reward(ctx, assertion_id)
    }
}
