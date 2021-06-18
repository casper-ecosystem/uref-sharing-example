# URef Sharing Example Contract

## Description

Example code to demonstrate a method of sharing access right URefs on the Casper Contract system.
Group access creation is used to ensure that only the desired accounts can call certain entrypoints.
In case an account makes a call to an entrypoint that it does not have access rights to,
the runtime will revert with an `InvalidContext` error.

For the retrieving of a URef into an account, two entrypoints are needed.
One with type `Contract` to get the URef from the contract.
Another one having `Session` type. This one stores the URef into the accounts storage context.
You need the ContractPackageHash of the contract to create access tokens, but you cannot create "recreate"
these at a later date, as that is considered `Forging` and results in an error named as such.

On creation of the access restricted contract we store the URefs into a separate contract,
that deals with the storage and sharing of the URefs. Then the user that was designated to have access rights
retrieves the access right URef from the storage contract. From that point on that user can use also
use the restricted entrypoint.

It is possible to serve this feature from the same contract as the one you wish to share URefs of/from.

- `share.rs`: standalone uref storage contract.
- `locked.rs`: testing contract.
- `locked_with_share.rs`: testing contract with the uref sharing feature integrated into it.

## make commands

Add wasm32-unknown-unknown target to the crate.
```bash
$ make prepare
```

Just builds the contracts.
```bash
$ make build-contract
```

Builds the code, copies `.wasm` files, then runs the tests. 
```bash
$ make test
```

Run rustfmt to format the code, then run clippy to ensure there is no best practices and warnings we missed.
```bash
$ make lint
```

## Deploy

See Casper documentation: [Deploying Contracts](https://docs.casperlabs.io/en/latest/dapp-dev-guide/deploying-contracts.html) and [Contracts on the Blockchain](https://docs.casperlabs.io/en/latest/dapp-dev-guide/calling-contracts.html).

## Contract entrypoints (Standalone edition)

### URef Sharing Contract

- `retrieve_uref`:
    - Arguments: None
    - Return: URef
    - Type: Contract
    - Description: Retrieves URef stored under callers `AccountHash`.

- `store_uref`:
    - Arguments:
        - `account_pubkey` - PublicKey
        - `uref` - URef
    - Return: None
    - Type: Contract
    - Description: Stores a `URef` in the contract under the `AccountHash` derived from the provided PublicKey.
    If there is a URef already stored to this account, the one stored will be overwritten with the new one.

### Locked Contract

- `get_access`:
    - Arguments:
        - `share_contract` - URef
    - Return: None
    - Type: Session
    - Description: Fetches access URef from `share_contract` and stores it in the callers account storage.

- `group_access_only`:
    - Arguments: None
    - Type: Contract
    - Description: Reverts with `777` user error. Only callable with access.



## Contract entrypoints (Integrated edition)

- `retrieve_urefs`:
    - Arguments: None
    - Return: Vec<URef>
    - Type: Contract
    - Description: Retrieves URefs stored under callers `AccountHash`.

- `append_urefs`:
    - Arguments:
        - `account_pubkeys` - Vec<PublicKey>
        - `urefs` - Vec<URef>
    - Return: None
    - Type: Contract
    - Description: Stores the `URef`s in the contract under the `AccountHash`es derived from the provided PublicKeys.
    Each individual account gets a uref, in the order both lists are supplied.

- `get_access`:
    - Arguments:
        - `this_contract` - URef
    - Return: None
    - Type: Session
    - Description: Fetches access URefs from `this_contract` via the `retrieve_urefs` entrypoint
    and stores them in the callers account storage, under themselves as their key.

- `group_access_only`:
    - Arguments: None
    - Type: Contract
    - Description: Reverts with `777` user error. Only callable with access.