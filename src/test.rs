#![cfg(test)]

use super::*;
use soroban_sdk::{Env, Address, testutils::Address as _};
use types::{aqua_pool_trait::AquaPoolTrait, price_feed_trait::PriceFeedTrait};

#[contract]
struct MockPoolContract;

#[contractimpl]
impl AquaPoolTrait for MockPoolContract {
    fn get_total_shares(e: Env) -> u128 {
        e.storage().instance().get(&"total_shares").unwrap()
    }

    fn get_tokens(e: Env) -> Vec<Address> {
        e.storage().instance().get(&"tokens").unwrap()
    }

    fn get_reserves(e: Env) -> Vec<u128> {
        e.storage().instance().get(&"reserves").unwrap()
    }
}

#[contractimpl]
impl MockPoolContract {
    pub fn init_pool(e: Env, total_shares: u128, tokens: Vec<Address>, reserves: Vec<u128>) {
        e.storage().instance().set(&"total_shares", &total_shares);
        e.storage().instance().set(&"tokens", &tokens);
        e.storage().instance().set(&"reserves", &reserves);
    }
}

#[contract]
struct MockOracleContract;

#[contractimpl]
impl PriceFeedTrait for MockOracleContract {
    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        e.storage().instance().get(&asset)
    }

    fn decimals(_e: Env) -> u32 {
        14
    }
}

#[contractimpl]
impl MockOracleContract {
    pub fn set_price(e: Env, asset: Asset, price: PriceData) {
        e.storage().instance().set(&asset, &price);
    }
}



#[test]
fn test() {
    let env = Env::default();

    env.mock_all_auths();

    let asset1 = Address::generate(&env);
    let asset2 = Address::generate(&env);

    let oracle_address = Address::generate(&env);
    env.register_contract(&oracle_address, MockOracleContract);
    let oracle_client = MockOracleContractClient::new(&env, &oracle_address);
    oracle_client.set_price(&Asset::Stellar(asset1.clone()), &PriceData { price: 10i128.pow(14), timestamp: 1000 });
    oracle_client.set_price(&Asset::Stellar(asset2.clone()), &PriceData { price: 10i128.pow(14), timestamp: 1000 });


    let pool_asset = Address::generate(&env);
    let pool_address = Address::generate(&env);
    env.register_contract(&pool_address, MockPoolContract);
    let pool_client = MockPoolContractClient::new(&env, &pool_address);
    pool_client.init_pool(&10u128.pow(7), &Vec::from_array(&env, [asset1.clone(), asset2.clone()]), &Vec::from_array(&env,[10u128.pow(7), 10u128.pow(7)]));

    let admin = Address::generate(&env);
    let proxy_address = Address::generate(&env);
    env.register_contract(&proxy_address, ProxyContract);
    let proxy_client = ProxyContractClient::new(&env, &proxy_address);
    proxy_client.config(&ConfigData {
        admin,
        oracle: oracle_address.clone(),
        pools: Map::from_array(&env,[(pool_asset.clone(), pool_address)]),
    });

    assert_eq!(proxy_client.lastprice(&Asset::Stellar(asset1.clone())).unwrap().price, 10i128.pow(14));

    assert_eq!(proxy_client.lastprice(&Asset::Stellar(asset2.clone())).unwrap().price, 10i128.pow(14));

    assert_eq!(proxy_client.decimals(), 14);

    assert_eq!(proxy_client.lastprice(&Asset::Stellar(Address::generate(&env))), None);

    let price = proxy_client.lastprice(&Asset::Stellar(pool_asset));

    assert_eq!(price.unwrap().price, 10i128.pow(14) * 2);
}
