use soroban_sdk::{contractclient, Env, Address, Vec};

// Aqua Pool Trait
#[contractclient(name = "AquaPoolClient")]
#[allow(dead_code)]
pub trait AquaPoolTrait {
    // Get the total number of shares
    fn get_total_shares(env: Env) -> u128;
    // Get pool token addresses
    fn get_tokens(env: Env) -> Vec<Address>;
    // Get the reserves of the pool
    fn get_reserves(env: Env) -> Vec<u128>;
}