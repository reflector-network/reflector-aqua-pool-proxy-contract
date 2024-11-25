#![no_std]

mod types;
mod extensions;

use extensions::env_extensions::EnvExtensions;
use soroban_sdk::{contract, contractimpl, Env, Vec, Address, U256, BytesN, Map};

use types::{aqua_pool_trait::AquaPoolClient, asset::Asset, config_data::ConfigData, error::Error, price_data::PriceData, price_feed_trait::{PriceFeedClient, PriceFeedTrait}};

const DECIMALS: u32 = 14;

#[contract]
pub struct ProxyContract;

#[contractimpl]
impl PriceFeedTrait for ProxyContract {
    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        let oracle_client = get_oracle_client(&e);
        let base: Asset = oracle_client.base();
        if base == asset {
            return Some(get_base_price(&e));
        }
        match asset {
            Asset::Stellar(address) => {
                let pool_address = e.get_pools().get(address.clone());
                if pool_address.is_some() {
                    return get_pool_price(&e, &pool_address.unwrap(), &oracle_client, &base);
                } else {
                    return oracle_client.lastprice(&Asset::Stellar(address));
                }
            }
            _ => {
                e.panic_with_error(Error::AssetNotSupported);
            }
        }
    }

    fn decimals(_e: Env) -> u32 {
        DECIMALS
    }

    fn base(e: Env) -> Asset {
        get_oracle_client(&e).base()
    }
}

#[contractimpl]
impl ProxyContract {
    
    // Initializes the contract with the provided configuration parameters. Can be invoked only once.
    //
    // # Arguments
    //
    // * `admin` - Admin account address
    // * `config` - Configuration parameters
    //
    // # Panics
    //
    // Panics if the contract is already initialized
    pub fn config(e: Env, config: ConfigData) {
        config.admin.require_auth();
        if e.is_initialized() {
            e.panic_with_error(Error::AlreadyInitialized);
        }
        e.set_admin(&config.admin);
        e.set_oracle(&config.oracle);
        e.set_pools(&config.pools);
    }

    // Sets the oracle contract address. Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `oracle` - Oracle contract address
    //
    // # Panics
    //
    // Panics if the caller is not the admin
    pub fn set_oracle(env: Env, oracle: Address) {
        env.panic_if_not_admin();
        env.set_oracle(&oracle);
    }

    // Adds pools to the supported pools list. Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `pools` - Pool addresses to add
    //
    // # Panics
    //
    // Panics if the caller is not the admin
    pub fn add_pools(env: Env, pools: Map<Address, Address>) {
        env.panic_if_not_admin();
        let mut current_pools = env.get_pools();
        for pool in pools.iter() {
            current_pools.set(pool.0, pool.1);
        }
        env.set_pools(&current_pools);
    }

    // Removes pools from the supported pools list. Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `pools` - Pool addresses to remove
    //
    // # Panics
    //
    // Panics if the caller is not the admin
    pub fn remove_pools(env: Env, pools: Vec<Address>) {
        env.panic_if_not_admin();
        let mut current_pools = env.get_pools();
        for pool in pools.iter() {
            current_pools.remove(pool);
        }
        env.set_pools(&current_pools);
    }

    

    // Updates the contract source code. Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `admin` - Admin account address
    // * `wasm_hash` - WASM hash of the contract source code
    //
    // # Panics
    //
    // Panics if the caller doesn't match admin address
    pub fn update_contract(env: Env, wasm_hash: BytesN<32>) {
        env.panic_if_not_admin();
        env.deployer().update_current_contract_wasm(wasm_hash)
    }
}

fn get_pool_price(e: &Env, pool: &Address, oracle_client: &PriceFeedClient, base: &Asset) -> Option<PriceData> {
    let pool_client = AquaPoolClient::new(e, pool);
    let total_shares = pool_client.get_total_shares();
    if total_shares == 0 {
        return None;
    }
    let tokens = pool_client.get_tokens();
    if tokens.len() != 2 {
        e.panic_with_error(Error::InvalidPool);
    }
    let reserves = pool_client.get_reserves();
    if reserves.len() != 2 {
        e.panic_with_error(Error::InvalidPool);
    }
    let token_a_price = get_price(e, oracle_client, base,&Asset::Stellar(tokens.first().unwrap()));
    let token_b_price = get_price(e, oracle_client, base,&Asset::Stellar(tokens.last().unwrap()));
    if token_a_price.is_none() || token_b_price.is_none() {
        return None;
    }
    calculate_price(e, reserves, total_shares, &token_a_price.unwrap(), &token_b_price.unwrap())
}

fn get_price(e: &Env, oracle_client: &PriceFeedClient, base: &Asset, asset: &Asset) -> Option<PriceData> {
    if base == asset {
        return Some(get_base_price(&e));
    } 
    oracle_client.lastprice(&asset)
}

fn get_base_price(e: &Env) -> PriceData {
    PriceData { price: 10i128.pow(DECIMALS), timestamp: e.ledger().timestamp() }
}

fn get_oracle_client<'a>(e: &Env) -> PriceFeedClient<'a> {
    let oracle = e.get_oracle();
    if oracle.is_none() {
        e.panic_with_error(Error::Unauthorized);
    }
    PriceFeedClient::new(&e, &oracle.unwrap())
}

fn calculate_price(e: &Env, reserves: Vec<u128>, total_shares: u128, token_a_price: &PriceData, token_b_price: &PriceData) -> Option<PriceData> {
    //cast values to U256 to avoid overflow
    let token_a_value = get_signle_token_reserves_value(e, reserves.first().unwrap(), token_a_price);
    let token_b_value = get_signle_token_reserves_value(e, reserves.last().unwrap(), token_b_price);
    let normalized_price = token_a_value.add(&token_b_value).div(&U256::from_u128(e, total_shares)).to_u128();
    if normalized_price.is_none() {
        return None;
    }
    Some(PriceData { price: normalized_price.unwrap() as i128, timestamp: token_a_price.timestamp })
}

fn get_signle_token_reserves_value(e: &Env, reserves: u128, token_price: &PriceData) -> U256 {
    let normalized_reserves = U256::from_u128(e, reserves);
    let normalized_price = U256::from_u128(e, token_price.price as u128);
    normalized_reserves.mul(&normalized_price)
}

mod test;
