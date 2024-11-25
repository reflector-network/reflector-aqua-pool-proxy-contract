use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// The error codes for the contract.
pub enum Error {
    // The contract is already initialized.
    AlreadyInitialized = 0,
    // The caller is not authorized to perform the operation.
    Unauthorized = 1,
    // Asset not supported.
    AssetNotSupported = 2,
    // Invalid pool.
    InvalidPool = 3,
}
