use anchor_lang::prelude::*;

#[error_code]
pub enum MoraError {
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
    #[msg("VLA is not enabled")]
    VlaDisabled,
    #[msg("VLA round is not in the correct phase")]
    InvalidVlaPhase,
    #[msg("Voting power too low to participate in VLA")]
    InsufficientVotingPower,
    #[msg("Commit hash mismatch")]
    CommitHashMismatch,
    #[msg("VLA round already exists")]
    VlaRoundAlreadyExists,
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

#[error_code]
pub enum VlaError {
    #[msg("Not in commit phase")]
    NotCommitPhase,
    #[msg("Not in reveal phase")]
    NotRevealPhase,
    #[msg("Already revealed")]
    AlreadyRevealed,
    #[msg("VLA round not closed")]
    VlaNotClosed,
    #[msg("VLA round already closed")]
    VlaAlreadyClosed,
}
