#![allow(non_upper_case_globals)]
use soroban_sdk::storage::Instance;
use soroban_sdk::{panic_with_error, Address, Env, Map};

use crate::types;

use types::error::Error;
const ADMIN_KEY: &str = "admin";
const ORACLE_KEY: &str = "oracle";
const POOLS_KEY: &str = "pools";

pub trait EnvExtensions {
    fn get_admin(&self) -> Option<Address>;

    fn set_admin(&self, admin: &Address);

    fn set_oracle(&self, oracle: &Address);

    fn get_oracle(&self) -> Option<Address>;

    fn set_pools(&self, pools: &Map<Address, Address>);

    fn get_pools(&self) -> Map<Address, Address>;

    fn panic_if_not_admin(&self);

    fn is_initialized(&self) -> bool;
}

impl EnvExtensions for Env {
    fn is_initialized(&self) -> bool {
        get_instance_storage(&self).has(&ADMIN_KEY)
    }

    fn get_admin(&self) -> Option<Address> {
        get_instance_storage(&self).get(&ADMIN_KEY)
    }

    fn set_admin(&self, admin: &Address) {
        get_instance_storage(&self).set(&ADMIN_KEY, admin);
    }

    fn set_oracle(&self, oracle: &Address) {
        get_instance_storage(&self).set(&ORACLE_KEY, oracle);
    }

    fn get_oracle(&self) -> Option<Address> {
        get_instance_storage(&self).get(&ORACLE_KEY)
    }

    fn set_pools(&self, pools: &Map<Address, Address>) {
        get_instance_storage(&self).set(&POOLS_KEY, pools);
    }

    fn get_pools(&self) -> Map<Address, Address> {
        get_instance_storage(&self).get(&POOLS_KEY).unwrap_or(Map::new(&self))
    }

    fn panic_if_not_admin(&self) {
        let admin = self.get_admin();
        if admin.is_none() {
            panic_with_error!(self, Error::Unauthorized);
        }
        admin.unwrap().require_auth()
    }
}

fn get_instance_storage(e: &Env) -> Instance {
    e.storage().instance()
}
