#![no_main]
extern crate alloc;

use std::convert::TryInto;

use contract::{
    contract_api::{runtime, runtime::get_named_arg, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    bytesrepr::{FromBytes, ToBytes},
    contracts::ContractPackageHash,
    CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
    PublicKey, URef,
};

/// Returns the list of the entry points in the contract with added group security.
pub fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        "retrieve_uref",
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "store_uref",
        vec![
            Parameter::new("uref".to_string(), CLType::URef),
            Parameter::new("account_pubkey".to_string(), CLType::PublicKey),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points
}

/// Deployer/upgrader function. Tries to retrieve any data presumably stored earlier
/// in the context associated to to `name`. If there is data, proceeds with that,
/// otherwise creates a new contract.
pub fn install_or_upgrade_contract(name: String) {
    let contract_package_hash: ContractPackageHash =
        match runtime::get_key(&format!("{}-package-hash", name)) {
            Some(contract_package_hash) => {
                contract_package_hash.into_hash().unwrap_or_revert().into()
            }
            None => {
                let (contract_package_hash, access_token) =
                    storage::create_contract_package_at_hash();
                runtime::put_key(
                    &format!("{}-package-hash", name),
                    contract_package_hash.into(),
                );
                // Store package hash wrapped so we can use it in the test context
                runtime::put_key(
                    &format!("{}-wrapped-package-hash", name),
                    storage::new_uref(contract_package_hash).into(),
                );
                runtime::put_key(&format!("{}-access-uref", name), access_token.into());
                contract_package_hash
            }
        };
    let entry_points = get_entry_points();
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points, Default::default());

    runtime::put_key(&name, contract_hash.into());
    runtime::put_key(
        &format!("{}-wrapped", name),
        storage::new_uref(contract_hash).into(),
    );
}

// Entry points

#[no_mangle]
fn retrieve_uref() {
    let uref: URef = get_key(&runtime::get_caller().to_string());
    runtime::ret(CLValue::from_t(uref).unwrap_or_revert())
}

#[no_mangle]
fn store_uref() {
    let user: PublicKey = get_named_arg("account_pubkey");
    let uref: URef = get_named_arg("uref");
    set_key(&user.to_account_hash().to_string(), uref);
}

// Utility functions

/// Getter function from context storage.
/// Returns the previously data previously stored under `name` key,
/// or returns the default value of the type expected at the end of the call.
fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

/// Creates new storage key `name` and stores `value` to it.
/// In case the key `name` already exists, overwrites it with the new data.
fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}

#[no_mangle]
fn call() {
    install_or_upgrade_contract(String::from("uref-share"));
}
