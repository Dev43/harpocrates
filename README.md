# Harpocrates

> Harpocrates (Ancient Greek: Ἁρποκράτης) was the god of silence, secrets and confidentiality in the Hellenistic religion developed in Ptolemaic Alexandria (and also an embodiment of hope, according to Plutarch).

## Motivation

## How to build and run it

To run this, first run `make` in the root of the project. This download the necessary dependencies and also set you up to be able to create and verify Zkproofs :).

Now, if you want this to be deployed to Arweave, you will need to have an arweave-keyfile with some AR on it. You can get a prefunded one [here](https://faucet.arweave.net/). Make sure to have the keyfile in the `fhe_contract` repository with the name `arweave-keyfile.json`.

Now all you have to do is go into the fhe_contract repository `fhe_contract` and run `cargo run -- run-all`. From there, just follow the necessary prompts!

## TODO

- [x] Create a suitable FHE application using sunscreen (POC) (voting)

- [x] Determine information that needs to be stored (JSON, tags etc)

- [x] Create CLI

  - [x] "deploy" contract to arweave - deploy should also deploy the init for the ZKsnark
  - [x] "fetch-latest" gets the latest state of the contract (simply the data)
  - [x] "compute-latest" runs through the local txns to compute the current state
  - [x] "vote" votes. or waits until there is another vote to add (also publishes a ZKsnark)

  TODO

  - [x] Run from top to bottom (brand new contract). Tag workable commits
  - [x] Add wallet connect to get each participant's eth address

### Stretch

- [x] Integrate walletconnect too (v1 && v2)
- [ ] Integrate Pinata
