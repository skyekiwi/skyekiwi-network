# SkyeKiwi Offchain VM Runtime in Cli
This crate is used by the `mock-enclave` to bridge transactions from the `pallet-s-contract` to the offchain VM and post the output back to the `pallet-parentchain` for off-chain syncing. But this crate along dose not handle any of the communication with the mainchain. 

## Usage
Currently calling the Cli interface will re-initialize the runtime everytime. We will transfrom it to continously listen to transactions over an local HTTPS port eventually. 

For now, it taks the following parameters: 
```rust
// the path to the dumped (unencrypted) state file, the state file will be updated once the transactions are executed.
state_file: Option<PathBuf>, 
// the trie root of state as of the latest execution
state_root: Option<String>,
// signer (executor) of the transactions; WIP
signer: Option<String>,
// a Borsh encoded hex string of the transactions
params: Option<String>,
// a flag of whether enable time tracking for testing/profiling
timings: bool,
```

A bit more on the `params`: it takes an array of transactions packed in borsh serialized binary form: 
```rust
struct InputParams {
    transaction_action: Option<String>,
    receiver: Option<String>,
    amount: Option<Balance>, // Balance is u128
    wasm_file: Option<PathBuf>,
    method: Option<String>,
    args: Option<String>,
    to: Option<String>,
}
```
For all transctions, `transaction_action` and `receiver` are always required. 

For `transaction_action = create_account`, an `amount` is required for the deposit into the new account. 


For `transaction_action = transfer`, an `amount` is required for the amount of token to be transfered into the receiver.


For `transaction_action = call`, the `receiver` is the account id of the destination contract to be called. `method` is the name of the method to be called. `args` is the arguments to be passed to the method in `serde_json` formated. `amount` is required (can be 0 but cannot be none) as the amount of deposit to send to methods marked with `#payble`. 


For `transaction_action = view_method_call`, the `receiver` is the account id of the destination contract to be called. `method` is the name of the method to be called. `args` is the arguments to be passed to the method in `serde_json` formated. 

For `transaction_action = deploy`, the `receiver` is the account id of the destination contract to be called. `wasm_file` is the source of the wasm to be deployed. `amount` is required for the inital deposit into the contract account for all the storage it takes. 



See `<root>/skw-tools/scripts/vm-interface-run.cjs` for usage. 