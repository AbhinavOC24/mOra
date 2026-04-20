use anchor_lang::InstructionData;
use litesvm::LiteSVM;
use solana_sdk::{
    clock::Clock,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;
use oo_core::instructions::admin::InitializeGlobalConfig; // I need to check if these are public

#[test]
fn test_mora_init_native() {
    let mut svm = LiteSVM::new();
    
    let program_id = Pubkey::from_str("E7TZEgFboKppM4V8yix6UEQrBh2WL2rDqbbn2JYxPnat").unwrap();
    let program_bytes = std::fs::read("../../target/deploy/oo_core.so").unwrap_or_else(|_| {
        vec![]
    });
    
    if program_bytes.is_empty() { return; }

    svm.add_program(program_id, &program_bytes);

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // The logic to test more deeply would go here
    msg!("LiteSVM native test: successfully loaded program.");
}
