use spl_associated_token_account::instruction::create_associated_token_account;
use swap2::{
    accounts::create_account_from_seed, jto::get_tip_instruction,
    swap::get_raydium_buy_swap_instruction,
};

use anyhow::Result;
use dotenv::dotenv;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::Instruction,
    message::{v0::Message, VersionedMessage},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::VersionedTransaction,
};
use spl_token::{
    instruction::{close_account, initialize_account},
    native_mint,
};
use std::{env, str::FromStr};

use solana_client::nonblocking::rpc_client::RpcClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Variables
    let token_pubkey: Pubkey = Pubkey::from_str("umgcPr2uQHzmCerCu6kSPBiaUdMWZewRRQmQ54Apump")?;
    let amount_sol_in: f64 = 0.1;
    let compute_unit_limit = 60_000;
    let compute_unit_price: u64 = 1_000_000;

    // Load Environment Variables
    dotenv().ok();
    let private_key: String = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set"); // Get private key from .env
    let rpc: String = env::var("RPC").expect("RPC must be set"); // Get RPC from .env

    // RPC Client
    let client: RpcClient = RpcClient::new_with_commitment(rpc, CommitmentConfig::confirmed()); // Create RPC Client

    // Keypair
    let keypair: Keypair = Keypair::from_base58_string(&private_key); // Create Keypair from private key

    // Compute Unit Limit - Try to adjust as close to the transaction cost as possible
    let cul: Instruction = ComputeBudgetInstruction::set_compute_unit_limit(compute_unit_limit); // 450 is the cost of a transfer

    // Compute Unit Price
    let cup: Instruction = ComputeBudgetInstruction::set_compute_unit_price(compute_unit_price);

    // Tip Amount
    let tip_amount: Option<u64> = Some(1_000_000);

    // Get a recent blockhash
    let recent_blockhash: Hash = client.get_latest_blockhash().await?;

    // Create a new account for WSOL
    let mut fund_amount = ((amount_sol_in * 1_000_000_000.0) as u64) + 4_000_000; // 4m lamports for rent
    if fund_amount < 15_000_000 {
        fund_amount = 15_000_000; // Minimum 15m lamports funding
    }
    let (seed, mint_pubkey) = create_account_from_seed(&keypair.pubkey());
    let caws: Instruction = system_instruction::create_account_with_seed(
        &keypair.pubkey(), // from_pubkey: sender wallet
        &mint_pubkey,      // to_pubkey: the account generated from the seed
        &keypair.pubkey(), // base: sender wallet
        &seed,             // seed: the seed string used to generate the account
        fund_amount,       // lamports: amount of lamports to fund the account with
        165,               // space: 165 in a Raydium swap
        &spl_token::id(),  // owner: token program
    );

    // Initialize the account
    let ia: Instruction = initialize_account(
        &spl_token::id(),   // token_program_id: token program
        &mint_pubkey,       // account_pubkey: the account generated from the seed (WSOL account)
        &native_mint::id(), // mint_pubkey: wsol token program
        &keypair.pubkey(),  // owner_pubkey: sender wallet
    )?;

    // Create the associated token account for SPL token
    let cata: Instruction = create_associated_token_account(
        &keypair.pubkey(), // funding_address: payer wallet
        &keypair.pubkey(), // wallet_address: sender wallet
        &token_pubkey,     // token_mint_address: (the SPL token's address)
        &spl_token::id(),  // token_program_id: token program
    );

    // Raydium Buy Swap Instructi
    let rs: Instruction = get_raydium_buy_swap_instruction(
        &keypair.pubkey(), // wallet_pubkey: sender wallet
        &mint_pubkey,      // mint_pubkey: the account generated from the seed (WSOL account)
        &token_pubkey,     // token_pubkey: the SPL token's address
        amount_sol_in,     // amount_sol_in: amount of SOL to spend
        0,                 // min_amount_out: minimum amount of SPL token to receive
    )
    .await?;
    // Close the WSOL account
    let ca: Instruction = close_account(
        &spl_token::id(),     // token_program_id: token program
        &mint_pubkey,         // account_pubkey: the account generated from the seed (WSOL account)
        &keypair.pubkey(),    // destination_pubkey: sender wallet
        &keypair.pubkey(),    // owner_pubkey: sender wallet
        &[&keypair.pubkey()], // signer_pubkey: sender wallet
    )?;

    // Jito Validator Tip Instruction
    let gti: Instruction = get_tip_instruction(&keypair.pubkey(), tip_amount);

    // Create the message
    let message: Message = Message::try_compile(
        &keypair.pubkey(),
        &[cul, cup, caws, ia, cata, rs, ca, gti],
        &[],
        recent_blockhash,
    )?;

    // Create a VersionedTransaction
    let transaction: VersionedTransaction = VersionedTransaction {
        message: VersionedMessage::V0(message.clone()), // Clone the message
        signatures: vec![keypair.sign_message(&message.serialize())],
    };

    // Send the transaction
    let signature: Signature = client.send_and_confirm_transaction(&transaction).await?;
    println!("Transaction signature: {}", signature);

    Ok(())
}
