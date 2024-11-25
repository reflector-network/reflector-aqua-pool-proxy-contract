use soroban_sdk::{contracttype, Address, Map};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]

// The configuration parameters for the contract.
pub struct ConfigData {
    // The admin address.
    pub admin: Address,
    // Supported pools. Pool token address -> pool contract address.
    pub pools: Map<Address, Address>,
    // Oracle contract address.
    pub oracle: Address,
}
