mod integrated;
mod standalone;

#[cfg(test)]
mod tests {
    // Standalone version tests
    use super::standalone::ShareContract;

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

    // Integrated version tests
    // These are the same tests as with the standalone version but the uref storage feature is written and
    // available from inside the same context.

    use super::integrated::IntegratedContarct;

    #[test]
    #[should_panic(expected = "ApiError::User(777)")]
    fn integrated_deployer_calls_function() {
        // Deploy contracts.
        let mut contract = IntegratedContarct::deploy();
        // Admin calls contract, has access rights to it, and so can
        contract.call_locked(&contract.admin.clone());
    }

    #[test]
    #[should_panic(expected = "InvalidContext")]
    fn integrated_unauth_calls_function() {
        // Deploy contracts.
        let mut contract = IntegratedContarct::deploy();

        // Call restricted function with user who does not hava access to it,
        // and so the call reverts with InvalidContext error.
        contract.call_locked(&contract.user.clone());
    }

    #[test]
    #[should_panic(expected = "ApiError::User(777)")]
    fn integrated_getting_access() {
        // Deploy contracts.
        let mut contract = IntegratedContarct::deploy();

        // User retrieves access rights to the function.
        contract.retrieve_urefs(&contract.user.clone());
        // User now has access rights to call the access restricted function.
        contract.call_locked(&contract.user.clone());

        // Test does not reach this unreachable,
        // since we managed to call the contract that reverts with User(777) error.
        unreachable!();
    }

    #[test]
    #[should_panic(expected = "User(1)")]
    fn integrated_unable_to_gain_access() {
        // Deploy contracts.
        let mut contract = IntegratedContarct::deploy();

        // User tries to retrieve access rights to the function.
        contract.retrieve_urefs(&contract.unauth.clone());

        // This user was not designated on deployment to recieve access rights,
        // so they recieve User(1) error, which means they would have gotten the "default" URef value.
        contract.call_locked(&contract.unauth.clone());
        unreachable!();
    }
}

fn main() {
    panic!("The main should not be used here");
}
