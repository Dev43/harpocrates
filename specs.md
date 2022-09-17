# Specs for the data to be processed

## Arweave

### Tags

For the tags, we need this information to be searchable. So the first tag

The contract ID is the hash of the contract data x unix_timestamp (not the best but will do)

```json
{
  "App-Name": "harpocrates", // app name can either be harpocrates-source for source code, or harpocrates-interactions for interactions
  "App-Version": "0.0.1",
  "Contract": "0x...",
  "Content-Type": "application/json",
  "Sunscreen-Version": "0.0.1",
  "Validity-Proof": "ZkSnark/circom@2.0.8/snarkjs@0.4.27",
  "Unix-Time": "134546456456",
  "Input": "{}"
}
```

#### Input

Note: it is assumed that if the argument says "\_\__enclosed_\_\_" we are talking about the data in the transaction.

- [x] Deploy `{"action":"deploy", arguments: [], validity_proof:"ID_OF_VALIDITY_PROOF"}`
- [ ] Vote `{"action":"vote", arguments: ["ID_OF_ARGUMENT_USED_OR_EMPTY_STRING", "\_\__enclosed_\_\_"], validity_proof:"ID_OF_VALIDITY_PROOF"}`
- [ ] Decrypt `{"action":"decrypt", arguments:[], validity_proof:"ID_OF_VALIDITY_PROOF"}`

### Data

Contract initial state: must be all set to 0 -> ZK proof to show that - or we simply let the new person come in and take care of that

```json
{
  "votes": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
}
```

then

```json
{
  "votes": [1, 0, 0, 0, 0, 0, 0, 0, 0, 0]
}
```
