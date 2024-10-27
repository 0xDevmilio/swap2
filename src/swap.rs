use std::str::FromStr;

use anyhow::Result;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address;

pub const RAYDIUM_LP_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const AMM: &str = "6BxoN7n1fMxT1azW3FhwMd88GDgueSgQkHdChjRMbjoE";
pub const AMM_AUTHORITY: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";
pub const AMM_OPEN_ORDERS: &str = "AjNByeJoUb1H5Zrae2zxiJArSz9BuF9TyD7N1Fg1XCjw";
pub const AMM_TARGET_ORDERS: &str = "AjNByeJoUb1H5Zrae2zxiJArSz9BuF9TyD7N1Fg1XCjw";
pub const POOL_COIN: &str = "8sN9Ed3Jmpd35wBoxqZxc2xeCMg28NG2nte9xRyVYkkm";
pub const POOL_PC: &str = "CiDFjcCH4QGML7UTq7TBXG8rnVeQNuDSwaJHwNdopBJM";
pub const SERUM_PROGRAM: &str = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";
pub const SERUM_MARKET: &str = "GD3tAeSiGMV3vNheU7tiGFtqRmerx9vQx5Du4pFTGSDx";
pub const SERUM_BIDS: &str = "FSbJGDRXsj14ZeTdzyUyQTWpdEqkRYYWLP4vpCrR2X14";
pub const SERUM_ASK: &str = "2amNMPYBYQV9kNJcUyuiunVRFfLeoExo6sEfYYAez755";
pub const SERUM_EVENT_QUEUE: &str = "DoMgH2m6Uhi17qYTbba2ryBJ5Dbw8P6aRCFSiyxt9Bfz";
pub const SERUM_COIN_VAULT: &str = "6yhdLpQd92CtC5FxQHyozRiSDvzSVCjHr3dBGV4WyKZf";
pub const SERUM_PC_VAULT: &str = "A27jMwMy1vzGi5QZLoCkN1VPaSEv3AntUr7dQyLJyfoB";
pub const SERUM_VAULT_SIGNER: &str = "2qBMFrCdaaX9rAU27rxemP1WYBxRt29tS966UtZf7Rdi";

/// Get the buy swap instruction for a Raydium swap.
pub async fn get_raydium_buy_swap_instruction(
    wallet_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    token_pubkey: &Pubkey,
    amount_sol_in: f64,
    min_amount_out: u64,
) -> Result<Instruction> {
    let accounts = vec![
        // SPL Token
        AccountMeta::new_readonly(Pubkey::from_str(TOKEN_PROGRAM)?, false),
        // AMM
        AccountMeta::new(Pubkey::from_str(AMM)?, false), // AmmId: address of the LP (FOUND IN TX)
        AccountMeta::new_readonly(Pubkey::from_str(AMM_AUTHORITY)?, false), // CONSTANT (ALSO FOUND IN TX)
        AccountMeta::new(Pubkey::from_str(AMM_OPEN_ORDERS)?, false),        // open orders
        AccountMeta::new(Pubkey::from_str(AMM_TARGET_ORDERS)?, false),      // target orders
        AccountMeta::new(Pubkey::from_str(POOL_COIN)?, false),              // base vault
        AccountMeta::new(Pubkey::from_str(POOL_PC)?, false),                // quote vault
        // Serum
        AccountMeta::new_readonly(Pubkey::from_str(SERUM_PROGRAM)?, false), // Serum program ID
        AccountMeta::new(Pubkey::from_str(SERUM_MARKET)?, false), // Serum market (marketId)
        AccountMeta::new(Pubkey::from_str(SERUM_BIDS)?, false),   // Serum bids
        AccountMeta::new(Pubkey::from_str(SERUM_ASK)?, false),    // Serum asks
        AccountMeta::new(Pubkey::from_str(SERUM_EVENT_QUEUE)?, false), // Serum event queue
        AccountMeta::new(Pubkey::from_str(SERUM_COIN_VAULT)?, false), // Serum base vault
        AccountMeta::new(Pubkey::from_str(SERUM_PC_VAULT)?, false), // Serum quote vault
        AccountMeta::new_readonly(Pubkey::from_str(SERUM_VAULT_SIGNER)?, false), // Vault signer (marketAuthority)
        // User
        AccountMeta::new(*mint_pubkey, false), // User source token account (WSOL account)
        AccountMeta::new(
            get_associated_token_address(&wallet_pubkey, &token_pubkey),
            false,
        ),
        // User destination account (minted account to hold SPL token)
        AccountMeta::new_readonly(*wallet_pubkey, true), // User source owner (sender wallet)
    ];

    let amount_in: u64 = (amount_sol_in * 1_000_000_000.0) as u64;
    let encoded_data = serialize_swap_instruction(amount_in, min_amount_out);

    let instruction = Instruction {
        program_id: Pubkey::from_str(RAYDIUM_LP_V4)?,
        accounts: accounts,
        data: encoded_data,
    };

    Ok(instruction)
}

/// Serialize the data for a buy swap instruction.
fn serialize_swap_instruction(amount_in: u64, minimum_amount_out: u64) -> Vec<u8> {
    let mut data = Vec::new();
    data.push(9); // Instruction identifier
    data.extend_from_slice(&amount_in.to_le_bytes());
    data.extend_from_slice(&minimum_amount_out.to_le_bytes());
    data
}
