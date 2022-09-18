# Harpocrates

![Harpocrates](harpocrates.jpg)

> Harpocrates (Ancient Greek: Ἁρποκράτης) was the god of silence, secrets and confidentiality in the Hellenistic religion developed in Ptolemaic Alexandria (and also an embodiment of hope, according to Plutarch).

## Warning

This repo is a POC, not meant to be in production. Use at your own risks.

## Motivation

Privacy is a human right.

Technology can be a double edge sword. It has made us extremely productive and inventive. It connects us like nothing else before. But this convenience and power can be used nefariously. We tend to forget that not so long ago, it was easy to be private. You would pay in cash, GPS didn't exist, cellphones less so. All these technologies degrade our option to be private. It's time to harness technology to swing the pendulum the other way.

I strongly identify with statement from Eric Hughes:

> "We must defend our own privacy if we expect to have any. We must come together and create systems which allow anonymous transactions to take place. People have been defending their own privacy for centuries with whispers, darkness, envelopes, closed doors, secret handshakes, and couriers. The technologies of the past did not allow for strong privacy, but electronic technologies do."

> > Eric Hughes: A Cypherpunk's Manifesto.

Now we have the tools (or at least we are very nearly there) to have the **optionality** to be private. Let's make sure we get there!

This project is not perfect, but I hope it will have an impact on people and spark their imagination on other similar privacy preserving projects.

## How to build and run it

To run this, first run `make` in the root of the project. This download the necessary dependencies and also set you up to be able to create and verify Zkproofs :).

Now, if you want this to be deployed to Arweave, you will need to have an arweave-keyfile with some AR on it. You can get a prefunded one [here](https://faucet.arweave.net/). Make sure to have the keyfile in the `fhe_contract` repository with the name `arweave-keyfile.json`.

Now all you have to do is go into the fhe_contract repository `fhe_contract` and run `cargo run -- run-all`. From there, just follow the necessary prompts!

More info can be found by running `cargo run -- --help`

```bash
USAGE:
    fhe_contract [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    compute-latest
    create-new-user
    deploy
    fetch-latest
    fetch-zk
    help               Print this message or the help of the given subcommand(s)
    init-state
    init-zk-proof
    run-all
    vote
```

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
