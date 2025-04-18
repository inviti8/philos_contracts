#![cfg(test)]
extern crate std;

use crate::{CollectiveContract, CollectiveContractClient};
use crate::{token};
use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, String, Symbol, FromVal
};

mod philos_node_token {
    soroban_sdk::contractimport!(
        file = "../philos-node-deployer/philos-node-token/target/wasm32-unknown-unknown/release/philos_node_token.optimized.wasm"
    );
}

mod philos_ipfs_token {
    soroban_sdk::contractimport!(
        file = "../philos-ipfs-deployer/philos-ipfs-token/target/wasm32-unknown-unknown/release/philos_ipfs_token.optimized.wasm"
    );
}

mod opus_token {
    soroban_sdk::contractimport!(
        file = "../opus_token/target/wasm32-unknown-unknown/release/opus_token.optimized.wasm"
    );
}

fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &sac.address()),
        token::StellarAssetClient::new(e, &sac.address()),
    )
}

#[test]
fn test_member_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let heavymeta: Symbol = Symbol::new(&env, "HVYM");
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let pay_token_admin_client  = pay_token.1;

    pay_token_admin_client.mint(&issuer, &10000);
    
    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    collective.fund_contract(&issuer, &500);

    let person1 = Address::generate(&env);
    let person2 = Address::generate(&env);
    let person3 = Address::generate(&env);

    pay_token_admin_client.mint(&person1, &100);
    pay_token_admin_client.mint(&person2, &100);
    pay_token_admin_client.mint(&person3, &50);

    collective.launch_opus(&888);


    let a = collective.join(&person1);
    collective.join(&person2);

    assert_eq!(collective.symbol(), heavymeta);

    assert_eq!(collective.join_fee(), 7);

    assert_eq!(collective.mint_fee(), 7);

    assert_eq!(pay_token_client.balance(&admin), 0);

    assert_eq!(a.address, person1);

    assert_eq!(collective.is_member(&person1), true);

    assert_eq!(collective.member_paid(&person1), 7);

    assert_eq!(collective.withdraw(&admin), true);

    assert_eq!(pay_token_client.balance(&admin), 514);

    collective.join(&person3);
    assert_eq!(collective.is_member(&person3), true);
    assert_eq!(collective.remove(&person3), true);
    assert_eq!(collective.is_member(&person3), false);

    assert_eq!(collective.update_join_fee(&21_u32), 21);
    assert_eq!(collective.join_fee(), 21);


}

#[test]
fn test_node_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let pay_token_admin_client  = pay_token.1;
    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    let name = String::from_val(&env, &"name");
    let symbol = String::from_val(&env, &"HVYMNODE");
    let descriptor = String::from_val(&env, &"DESCRIPTOR");

    let person1 = Address::generate(&env);

    pay_token_admin_client.mint(&person1, &100);
    collective.join(&person1);

    let contract_id = collective.deploy_node_token(&person1, &name, &descriptor);

    // Invoke contract to check that it is initialized.
    let token = philos_node_token::Client::new(&env, &contract_id);
    assert_eq!(token.balance(&person1), 1);
    assert_eq!(token.name(), name);
    assert_eq!(token.symbol(), symbol);
    assert_eq!(token.descriptor(), descriptor);

    
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_node_creation_fail() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    let name = String::from_val(&env, &"name");
    let descriptor = String::from_val(&env, &"DESCRIPTOR");

    let person1 = Address::generate(&env);

    collective.deploy_node_token(&person1, &name, &descriptor);

    
}

#[test]
fn test_ipfs_token_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let pay_token_admin_client  = pay_token.1;
    pay_token_admin_client.mint(&issuer, &1000);

    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    collective.fund_contract(&issuer, &500);
    collective.launch_opus(&888);

    let ledger = env.ledger();
    let name = String::from_val(&env, &"name");
    let symbol = String::from_val(&env, &"HVYMFILE");
    let ipfs_hash = String::from_val(&env, &"IPFS_HASH");
    let file_type = String::from_val(&env, &"FILE_TYPE");
    let published = ledger.timestamp();
    let gateways = String::from_val(&env, &"GATEWAYS");
    let _ipns_hash: Option<String> = None;

    let person1 = Address::generate(&env);

    pay_token_admin_client.mint(&person1, &100);
    collective.join(&person1);

    let contract_id = collective.deploy_ipfs_token(&person1, &name, &ipfs_hash, &file_type, &gateways, &_ipns_hash);

    // Invoke contract to check that it is initialized.
    let token = philos_ipfs_token::Client::new(&env, &contract_id);
    token.mint(&person1, &1000);
    assert_eq!(token.name(), name);
    assert_eq!(token.symbol(), symbol);
    assert_eq!(token.ipfs_hash(&person1), ipfs_hash);
    assert_eq!(token.file_type(&person1), file_type);
    assert_eq!(token.published(&person1), published);
    assert_eq!(token.gateways(&person1), gateways);
    assert_eq!(token.ipns_hash(&person1), _ipns_hash);
    //check rewards
    let opus = opus_token::Client::new(&env, &collective.opus_address());
    assert_eq!(opus.balance(&person1), 7);

    
}

#[test]
#[should_panic(expected = "network not up")]
fn test_ipfs_token_creation_fail_no_network() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let pay_token_admin_client  = pay_token.1;
    pay_token_admin_client.mint(&issuer, &1000);

    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    collective.fund_contract(&issuer, &500);

    let name = String::from_val(&env, &"name");
    let ipfs_hash = String::from_val(&env, &"IPFS_HASH");
    let file_type = String::from_val(&env, &"FILE_TYPE");
    let gateways = String::from_val(&env, &"GATEWAYS");
    let _ipns_hash: Option<String> = None;

    let person1 = Address::generate(&env);

    pay_token_admin_client.mint(&person1, &100);
    collective.join(&person1);

    collective.deploy_ipfs_token(&person1, &name, &ipfs_hash, &file_type, &gateways, &_ipns_hash);

    
}

#[test]
#[should_panic(expected = "opus already up")]
fn test_fail_launch_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let pay_token_admin_client  = pay_token.1;
    pay_token_admin_client.mint(&issuer, &1000);
    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    collective.fund_contract(&issuer, &500);
    collective.launch_opus(&888);
    collective.launch_opus(&888);
    
}

#[test]
#[should_panic(expected = "not enough to cover fee")]
fn test_ipfs_token_creation_fail_no_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let pay_token_admin_client  = pay_token.1;
    pay_token_admin_client.mint(&issuer, &1000);
    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    collective.fund_contract(&issuer, &500);
    collective.launch_opus(&888);

    let name = String::from_val(&env, &"name");
    let ipfs_hash = String::from_val(&env, &"IPFS_HASH");
    let file_type = String::from_val(&env, &"FILE_TYPE");
    let gateways = String::from_val(&env, &"GATEWAYS");
    let _ipns_hash: Option<String> = None;

    let person1 = Address::generate(&env);
    

    pay_token_admin_client.mint(&person1, &7);
    assert_eq!(pay_token_client.balance(&person1), 7);
    collective.join(&person1);
    assert_eq!(pay_token_client.balance(&person1), 0);

    collective.deploy_ipfs_token(&person1, &name, &ipfs_hash, &file_type, &gateways, &_ipns_hash);

    
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_ipfs_token_creation_fail_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let pay_token_admin_client  = pay_token.1;
    pay_token_admin_client.mint(&issuer, &1000);
    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    collective.fund_contract(&issuer, &500);
    collective.launch_opus(&888);

    let name = String::from_val(&env, &"name");
    let ipfs_hash = String::from_val(&env, &"IPFS_HASH");
    let file_type = String::from_val(&env, &"FILE_TYPE");
    let gateways = String::from_val(&env, &"GATEWAYS");
    let _ipns_hash: Option<String> = None;

    let person1 = Address::generate(&env);

    pay_token_admin_client.mint(&person1, &100);

    collective.deploy_ipfs_token(&person1, &name, &ipfs_hash, &file_type, &gateways, &_ipns_hash);


    
}

#[test]
#[should_panic(expected = "not enough to cover fee")]
fn test_member_join_fail() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let pay_token_admin_client  = pay_token.1;
    //pay_token_admin_client.mint(&admin, &1000);
    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    let person1 = Address::generate(&env);

    pay_token_admin_client.mint(&person1, &5);

    collective.join(&person1);


}

#[test]
#[should_panic(expected = "already part of collective")]
fn test_member_join_twice_fail() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let pay_token_admin_client  = pay_token.1;
    //pay_token_admin_client.mint(&admin, &1000);
    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    let person1 = Address::generate(&env);

    pay_token_admin_client.mint(&person1, &10);

    collective.join(&person1);
    collective.join(&person1);


}

#[test]
#[should_panic(expected = "not enough collected to withdraw")]
fn test_withdraw_fail() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let pay_token = create_token_contract(&env, &admin);
    let pay_token_client = pay_token.0;
    let collective = CollectiveContractClient::new(&env, &env.register(CollectiveContract, (&admin, 7_u32, 7_u32, &pay_token_client.address, 7_u32)));

    collective.withdraw(&admin);


}
