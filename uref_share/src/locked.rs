#![no_main]
extern crate alloc;

use contract::{
    contract_api::{runtime, runtime::revert, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    contracts::ContractPackageHash, runtime_args, ApiError, CLType, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Key, Parameter, PublicKey, RuntimeArgs, URef,
};

pub fn prepare_access(contract_package_hash: &ContractPackageHash) {
    // Get list of public keys of the potential admins
    let users: Vec<PublicKey> = runtime::get_named_arg("users");
    // Get the package hash for the uref share contract
    let share_contract: ContractPackageHash = runtime::get_named_arg("share_hash");

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

    for user in users {
        let _: () = runtime::call_versioned_contract(
            share_contract,
            None,
            "store_uref",
            runtime_args! {"uref" => admin_group.pop().unwrap_or_revert(), "account_pubkey" => user},
        );
    }
}

/// Returns the list of the entry points in the contract with added group security.
pub fn get_entry_points(contract_package_hash: &ContractPackageHash) -> EntryPoints {
    prepare_access(contract_package_hash);
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        "group_access_only",
        vec![],
        CLType::URef,
        EntryPointAccess::groups(&["admin"]),
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "get_access",
        vec![Parameter::new("share_contract".to_string(), CLType::URef)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
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

    let entry_points = get_entry_points(&contract_package_hash);
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
fn get_access() {
    let share_contract: ContractPackageHash = runtime::get_named_arg("share_contract");

    let access: URef =
        runtime::call_versioned_contract(share_contract, None, "retrieve_uref", runtime_args! {});

    if access == URef::default() {
        revert(ApiError::User(1));
    }

    runtime::put_key("admin", Key::URef(access));
}

#[no_mangle]
fn group_access_only() {
    // JACKPOT revert with User error 777 to see without a doubt that we have access to this function.
    revert(ApiError::User(777))
}

#[no_mangle]
fn call() {
    install_or_upgrade_contract(String::from("locked"));
}
