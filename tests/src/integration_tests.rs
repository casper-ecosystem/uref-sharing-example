#[cfg(test)]
mod tests {
    use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
    use casper_types::{
        account::AccountHash, runtime_args, ContractPackageHash, PublicKey, RuntimeArgs, SecretKey,
        U512,
    };

    pub struct ShareContract {
        pub context: TestContext,
        pub locked_hash: Hash,
        pub package_hash: ContractPackageHash,
        pub admin: AccountHash,
        pub user: AccountHash,
        pub user_pk: PublicKey,
        pub unauth: AccountHash,
    }

    impl ShareContract {
        pub fn deploy() -> Self {
            // Create admin.
            let admin_key: PublicKey = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap().into();
            let admin_addr = AccountHash::from(&admin_key);

            // Create plain user.
            let user_key: PublicKey = SecretKey::ed25519_from_bytes([2u8; 32]).unwrap().into();
            let user_addr = AccountHash::from(&user_key);

            // Create plain user, will not receive access rights.
            let unauth_key: PublicKey = SecretKey::ed25519_from_bytes([3u8; 32]).unwrap().into();
            let unauth_addr = AccountHash::from(&unauth_key);

            // Create context.
            let mut context = TestContextBuilder::new()
                .with_public_key(admin_key, U512::from(500_000_000_000_000_000u64))
                .with_public_key(user_key.clone(), U512::from(500_000_000_000_000_000u64))
                .with_public_key(unauth_key, U512::from(500_000_000_000_000_000u64))
                .build();

            // Deploy the URef sharing contract onto the context.
            let session_code = Code::from("uref-share.wasm");
            let session = SessionBuilder::new(session_code, RuntimeArgs::new())
                .with_address(admin_addr)
                .with_authorization_keys(&[admin_addr])
                .build();
            context.run(session);

            // Get sharing contract hash
            let package_hash = context
                .query(admin_addr, &["uref-share-wrapped-package-hash".to_string()])
                .unwrap_or_else(|_| panic!("uref-share-wrapped-package-hash contract not found"))
                .into_t()
                .unwrap_or_else(|_| panic!("uref-share-wrapped-package-hash has wrong type"));

            // Get the testing contract onto the context
            let locked_code = Code::from("locked.wasm");
            let locked = SessionBuilder::new(
                locked_code,
                runtime_args! {
                    "users"=> vec![user_key.clone()],
                    "share_hash"=> package_hash
                },
            )
            .with_address(admin_addr)
            .with_authorization_keys(&[admin_addr])
            .build();
            context.run(locked);

            // Get the hash for the package testing contract
            let locked_hash = context
                .query(admin_addr, &["locked-wrapped".to_string()])
                .unwrap_or_else(|_| panic!("locked contract not found"))
                .into_t()
                .unwrap_or_else(|_| panic!("locked has wrong type"));

            Self {
                context,
                locked_hash,
                package_hash,
                admin: admin_addr,
                user: user_addr,
                user_pk: user_key,
                unauth: unauth_addr,
            }
        }

        /// Call the access restricted function on the testing contract.
        pub fn call_locked(&mut self, caller: &AccountHash) {
            let session_code = Code::Hash(self.locked_hash, "group_access_only".to_string());
            let session = SessionBuilder::new(session_code, runtime_args! {})
                .with_address(*caller)
                .with_authorization_keys(&[*caller])
                .build();
            self.context.run(session);
        }

        /// Call the function that gets the user rights to call the access restricted function.
        pub fn get_access(&mut self, caller: &AccountHash) {
            let session_code = Code::Hash(self.locked_hash, "get_access".to_string());
            let session = SessionBuilder::new(
                session_code,
                runtime_args! {
                    "share_contract" => self.package_hash
                },
            )
            .with_address(*caller)
            .with_authorization_keys(&[*caller])
            .build();
            self.context.run(session);
        }
    }

    #[test]
    #[should_panic(expected = "ApiError::User(777)")]
    fn deployer_calls_function() {
        // Deploy contracts.
        let mut contract = ShareContract::deploy();
        // Admin calls contract, has access rights to it, and so can
        contract.call_locked(&contract.admin.clone());
    }

    #[test]
    #[should_panic(expected = "InvalidContext")]
    fn unauth_calls_function() {
        // Deploy contracts.
        let mut contract = ShareContract::deploy();

        // Call restricted function with user who does not hava access to it,
        // and so the call reverts with InvalidContext error.
        contract.call_locked(&contract.user.clone());
    }

    #[test]
    #[should_panic(expected = "ApiError::User(777)")]
    fn getting_access() {
        // Deploy contracts.
        let mut contract = ShareContract::deploy();

        // User retrieves access rights to the function.
        contract.get_access(&contract.user.clone());

        // User now has access rights to call the access restricted function.
        contract.call_locked(&contract.user.clone());

        // Test does not reach this unreachable,
        // since we managed to call the contract that reverts with User(777) error.
        unreachable!();
    }

    #[test]
    #[should_panic(expected = "User(1)")]
    fn unable_to_gain_access() {
        // Deploy contracts.
        let mut contract = ShareContract::deploy();

        // User tries to retrieve access rights to the function.
        contract.get_access(&contract.unauth.clone());

        // This user was not designated on deployment to recieve access rights,
        // so they recieve User(1) error, which means they would have gotten the "default" URef value.
        contract.call_locked(&contract.unauth.clone());
        unreachable!();
    }
}

fn main() {
    panic!("The main should not be used here");
}
