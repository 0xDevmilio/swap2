use rand::{distributions::Alphanumeric, Rng};
use solana_sdk::pubkey::Pubkey;

/// Create an account using a randomly generated seed.
pub fn create_account_from_seed(base_pubkey: &Pubkey) -> (String, Pubkey) {
    // Generate a random seed
    let seed: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32) // Length of the seed string
        .map(char::from)
        .collect();

    // Derive the new public key
    let to_pubkey: Pubkey = Pubkey::create_with_seed(base_pubkey, &seed, &spl_token::id())
        .expect("Failed to create public key with seed");

    (seed, to_pubkey)
}
