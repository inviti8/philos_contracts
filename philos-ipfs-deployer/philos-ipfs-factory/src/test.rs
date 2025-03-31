#![cfg(test)]
extern crate alloc;
extern crate std;

use crate::{PhilosFactory, PhilosFactoryClient};
use alloc::vec;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, BytesN, Val, Env, FromVal, IntoVal, String, Vec,
};

// The contract that will be deployed by the deployer contract.
mod contract {
    soroban_sdk::contractimport!(
        file =
            "../philos-ipfs-token/target/wasm32-unknown-unknown/release/philos_ipfs_token.wasm"
    );
}

#[test]
fn test() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let deployer_client = PhilosFactoryClient::new(&env, &env.register(PhilosFactory, (&admin,)));
    let name = String::from_val(&env, &"name");
    let symbol = String::from_val(&env, &"symbol");
    let ipfs_hash = String::from_val(&env, &"IPFS_HASH");
    let file_type = String::from_val(&env, &"FILE_TYPE");
    let published = String::from_val(&env, &"PUBLISHED");
    let gateways = String::from_val(&env, &"GATEWAYS");
    let _ipns_hash: Option<String> = None;

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    // Deploy contract using deployer, and include an init function to call.
    let salt = BytesN::from_array(&env, &[0; 32]);
    let constructor_args: Vec<Val> = (admin.clone(), 5u32, name.clone(), symbol.clone(), ipfs_hash, file_type, published, gateways, _ipns_hash).into_val(&env);
    env.mock_all_auths();
    let contract_id = deployer_client.deploy(&wasm_hash, &salt, &constructor_args);

    // An authorization from the admin is required.
    let expected_auth = AuthorizedInvocation {
        // Top-level authorized function is `deploy` with all the arguments.
        function: AuthorizedFunction::Contract((
            deployer_client.address,
            symbol_short!("deploy"),
            (wasm_hash.clone(), salt, constructor_args).into_val(&env),
        )),
        sub_invocations: vec![],
    };
    assert_eq!(env.auths(), vec![(admin, expected_auth)]);

    // Invoke contract to check that it is initialized.
    let token = contract::Client::new(&env, &contract_id);
    assert_eq!(token.name(), name);
    assert_eq!(token.symbol(), symbol);
}
