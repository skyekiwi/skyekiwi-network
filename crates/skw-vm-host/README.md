# skw-vm-host

This crate exposes the host functions used by the SkyeKiwi Offchain VM modified from the host functions of the near runtime. 

Some noteable modifications:
1. Remove all pairing curve related code.
2. Remove staking, validator and epoch related funcs. 
3. Rename `block_height` or `block_index` to `block_number` while they should be bridged from the mainchain. 

Note, this logic assumes the little endian byte ordering of the memory used by the smart contract.
