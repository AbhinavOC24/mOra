use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Global {
    pub admin: Pubkey,
    pub min_proposer_bond: u64,
    pub arbiter_reward_bps: u16,
    pub burn_vault: Pubkey,
    pub min_arbiter_voting_power: u128,
    pub vla_commit_window: i64,
    pub vla_enabled: bool,
    pub quorum: u128,
}

#[account]
#[derive(InitSpace)]
pub struct LockingConfig {
    pub admin: Pubkey,
    pub mora_mint: Pubkey,
    pub min_lock_duration: i64,
    pub max_lock_duration: i64,
    pub total_slope: u128,
    pub total_bias: u128,
    pub last_update_ts: i64,
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
    pub proposed_at: Option<i64>,
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

#[account]
#[derive(InitSpace)]
pub struct VlaRound {
    pub assertion_id: u64,
    pub opened_at: i64,
    pub commit_ends_at: i64,
    pub reveal_ends_at: i64,
    pub status: VlaRoundStatus,
    pub total_power_disputer: u128,
    pub total_power_proposer: u128,
    pub winning_side: Option<VlaSide>,
}

#[account]
#[derive(InitSpace)]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub assertion_id: u64,
    pub commit_hash: [u8; 32],
    pub revealed: bool,
    pub side: Option<VlaSide>,
    pub voting_power: u128,
    pub reward_claimed: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug)]
pub enum VlaSide {
    Proposer,
    Disputer,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum VlaRoundStatus {
    CommitPhase,
    RevealPhase,
    Finalized,
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
    YesNo(bool),
    Number(i64),
    String(#[max_len(512)] String),
}
