## Port of NEAR VM

This VM is intended to be the secret runtime VM for the SkyeKiwi Network. Ported from the [NEAR Protocol](https://github.com/near/nearcore)

## Usage

Run `yarn build` will compile the contract into WASM blob and copy the result to the `wasm` folder of the root directory. 

Run `yarn start` to compile the contract and run vm. Keep in mind that the contract name and paramters are hardcoded for the `greeting` contract (for now).

Run `yarn vm` will write output from the VM to `result.json`.

Excuted With: 
```javascript

let state = {}

state = runVM({
  contextFile: './context/system.json',
  methodName: 'set_greeting',
  input: '{"message": "system_hello"}',
  stateInput: '{}',
})

state = runVM({
  contextFile: './context/bob.json',
  methodName: 'set_greeting',
  input: '{"message": "bob_hello"}',
  stateInput: JSON.stringify(state),
})

state = runVM({
  contextFile: './context/zs.json',
  methodName: 'set_greeting',
  input: '{"message": "zs_hello"}',
  stateInput: JSON.stringify(state),
})

state = runVM({
  methodName: 'get_greeting',
  input: '{"account_id": "bob.sk"}',
  stateInput: JSON.stringify(state),
})
```

It should outputs something like: 
```
$ npx ts-node ./scripts/vm.ts
$ yarn vm 
$ cd src/near-vm-runner-standalone && cargo build --release
    Finished release [optimized] target(s) in 0.18s
$ ./src/near-vm-runner-standalone/target/release/near-vm-runner-standalone --context-file ./context/system.json --wasm-file ./wasm/greeting.wasm --method-name set_greeting --input '{"message": "system_hello"}' --state '{}'  > result.json

-------EXEC RESULT BEGINS-------
{
  'a\t\x00\x00\x00system.sk': '\f\x00\x00\x00system_hello',
  STATE: '\x01\x00\x00\x00a'
}
------- EXEC RESULT ENDS -------

$ ./src/near-vm-runner-standalone/target/release/near-vm-runner-standalone --context-file ./context/bob.json --wasm-file ./wasm/greeting.wasm --method-name set_greeting --input '{"message": "bob_hello"}' --state '{"YQkAAABzeXN0ZW0uc2s=":"DAAAAHN5c3RlbV9oZWxsbw==","U1RBVEU=":"AQAAAGE="}'  > result.json

-------EXEC RESULT BEGINS-------
{
  STATE: '\x01\x00\x00\x00a',
  'a\x06\x00\x00\x00bob.sk': '\t\x00\x00\x00bob_hello',
  'a\t\x00\x00\x00system.sk': '\f\x00\x00\x00system_hello'
}
------- EXEC RESULT ENDS -------

$ ./src/near-vm-runner-standalone/target/release/near-vm-runner-standalone --context-file ./context/zs.json --wasm-file ./wasm/greeting.wasm --method-name set_greeting --input '{"message": "zs_hello"}' --state '{"U1RBVEU=":"AQAAAGE=","YQYAAABib2Iuc2s=":"CQAAAGJvYl9oZWxsbw==","YQkAAABzeXN0ZW0uc2s=":"DAAAAHN5c3RlbV9oZWxsbw=="}'  > result.json

-------EXEC RESULT BEGINS-------
{
  'a\t\x00\x00\x00system.sk': '\f\x00\x00\x00system_hello',
  'a\x06\x00\x00\x00bob.sk': '\t\x00\x00\x00bob_hello',
  STATE: '\x01\x00\x00\x00a',
  'a\x05\x00\x00\x00zs.sk': '\b\x00\x00\x00zs_hello'
}
------- EXEC RESULT ENDS -------

$ ./src/near-vm-runner-standalone/target/release/near-vm-runner-standalone --context-file ./context/system.json --wasm-file ./wasm/greeting.wasm --method-name get_greeting --input '{"account_id": "bob.sk"}' --state '{"YQkAAABzeXN0ZW0uc2s=":"DAAAAHN5c3RlbV9oZWxsbw==","YQYAAABib2Iuc2s=":"CQAAAGJvYl9oZWxsbw==","U1RBVEU=":"AQAAAGE=","YQUAAAB6cy5zaw==":"CAAAAHpzX2hlbGxv"}'  > result.json

-------EXEC RESULT BEGINS-------
Return Value "bob_hello"
{
  'a\x06\x00\x00\x00bob.sk': '\t\x00\x00\x00bob_hello',
  STATE: '\x01\x00\x00\x00a',
  'a\x05\x00\x00\x00zs.sk': '\b\x00\x00\x00zs_hello',
  'a\t\x00\x00\x00system.sk': '\f\x00\x00\x00system_hello'
}
------- EXEC RESULT ENDS -------
```


## License

The entire code within this repository is licensed under the [GPLv3](LICENSE).

Please [contact us](https://skye.kiwi) if you have questions about
the licensing of our products.
