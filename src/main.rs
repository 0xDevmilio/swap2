use swap2::{accounts::create_account_from_seed, jto::get_tip_instruction};

use anyhow::Result;
use dotenv::dotenv;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::Instruction,
    message::{v0::Message, VersionedMessage},
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::VersionedTransaction,
};
use spl_token::{
    instruction::{close_account, initialize_account},
    native_mint,
};
use std::env;

use solana_client::nonblocking::rpc_client::RpcClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Variables
    let amount_sol_in: f64 = 0.1;
    let compute_unit_limit = 8_000;
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
        &[cul, cup, caws, ia, ca, gti],
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
