#![no_main]
extern crate alloc;

use std::convert::TryInto;

use contract::{
    contract_api::{
        runtime,
        runtime::{get_named_arg, revert},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    bytesrepr::{FromBytes, ToBytes},
    contracts::ContractPackageHash,
    runtime_args, ApiError, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType,
    EntryPoints, Key, Parameter, PublicKey, RuntimeArgs, URef,
};

pub fn prepare_access(contract_package_hash: &ContractPackageHash) -> (Vec<PublicKey>, Vec<URef>){
    // Get list of public keys of the potential admins
    let users: Vec<PublicKey> = runtime::get_named_arg("users");

    let mut admin_group = storage::create_contract_user_group(
        *contract_package_hash,
        "admin",
        (users.len() + 1) as u8,
        alloc::collections::BTreeSet::default(),
    )
    .unwrap_or_revert();

    runtime::put_key(
        "locked_deployer_admin_access",
        Key::URef(admin_group.pop().unwrap_or_revert()),
    );
    (users, admin_group)
}

/// Returns the list of the entry points in the contract with added group security.
pub fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        "group_access_only",
        vec![],
        CLType::Unit,
        EntryPointAccess::groups(&["admin"]),
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "get_access",
        vec![Parameter::new(
            "urefs".to_string(),
            CLType::List(Box::new(CLType::URef)),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "retrieve_urefs",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "append_urefs",
        vec![
            Parameter::new("urefs".to_string(), CLType::List(Box::new(CLType::URef))),
            Parameter::new("account_pubkeys".to_string(), CLType::List(Box::new(CLType::PublicKey))),
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
    let (users, admin_group) = prepare_access(&contract_package_hash);
    let _: () = runtime::call_versioned_contract(
        contract_package_hash,
        None,
        "append_urefs",
        runtime_args! {"urefs" => admin_group, "account_pubkeys" => users},
    );
    runtime::put_key(&name, contract_hash.into());
    runtime::put_key(
        &format!("{}-wrapped-hash", name),
        storage::new_uref(contract_hash).into(),
    );
}

// Entry points

#[no_mangle]
fn append_urefs() {
    let urefs: Vec<URef> = get_named_arg("urefs");
    let mut users: Vec<PublicKey> = get_named_arg("account_pubkeys");
    if urefs.len()!=users.len(){
        revert(ApiError::User(3));
    }

    for uref in urefs{
        let user_key = users.pop().unwrap_or_revert()
            .to_account_hash()
            .to_string();
        let mut personal_uref_list: Vec<URef> = get_key(&user_key);
        personal_uref_list.push(uref);
        set_key(&user_key, personal_uref_list);
    }
}

#[no_mangle]
fn retrieve_urefs() {
    let urefs: Vec<URef> = get_key(&runtime::get_caller().to_string());
    if urefs.is_empty() {
        revert(ApiError::User(1));
    }
    let _: () = runtime::call_versioned_contract(
        get_key("locked-with-share-package-hash"),
        None,
        "get_access",
        runtime_args! {"urefs" => urefs},
    );
}

#[no_mangle]
fn get_access() {
    let mut urefs: Vec<URef> = runtime::get_named_arg("urefs");
    if urefs.is_empty() {
        revert(ApiError::User(2));
    }
    let mut my_keys: Vec<URef> = get_key("my_access_keys");
    my_keys.append(&mut urefs);
    set_key("my_access_keys", my_keys)
}

#[no_mangle]
fn group_access_only() {
    // JACKPOT revert with User error 777 to see without a doubt that we have access to this function.
    revert(ApiError::User(777))
}

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
    install_or_upgrade_contract(String::from("locked-with-share"));
}
