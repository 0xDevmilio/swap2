use rand::{thread_rng, Rng};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_instruction};
use std::str::FromStr;

// Tip instruction
pub fn get_tip_instruction(payer_pubkey: &Pubkey, tip_amount: Option<u64>) -> Instruction {
    // 8 Jito Validator Tip Addresses
    pub const TIP_ADDRESSES: [&str; 8] = [
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
        "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
        "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
        "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
        "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
        "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
        "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
        "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
    ];

    // Random generator
    let mut rng = thread_rng();

    // Random tip address selector
    let tip_address: Pubkey =
        Pubkey::from_str(TIP_ADDRESSES[rng.gen_range(0..TIP_ADDRESSES.len())])
            .expect("must be a valid pubkey");

    // Tip amount
    let amt = if tip_amount.is_some() {
        tip_amount.unwrap()
    } else {
        10_000_000
    };

    // Transfer instruction
    system_instruction::transfer(&payer_pubkey, &tip_address, amt)
}
