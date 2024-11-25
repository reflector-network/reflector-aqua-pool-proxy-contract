use soroban_sdk::{contractclient, Env};

use super::{price_data::PriceData, asset::Asset};

// Oracle feed interface description
#[contractclient(name = "PriceFeedClient")]
pub trait PriceFeedTrait {
    // Get the most recent price for an asset
    fn lastprice(env: Env, asset: Asset) -> Option<PriceData>;
    
    fn decimals(env: Env) -> u32;

    fn base(env: Env) -> Asset;
}