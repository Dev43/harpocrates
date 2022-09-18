# 🤐 Harpocrates 🙊

![Harpocrates](harpocrates.jpg)

> Harpocrates (Ancient Greek: Ἁρποκράτης) was the god of silence, secrets and confidentiality in the Hellenistic religion developed in Ptolemaic Alexandria (and also an embodiment of hope, according to Plutarch).

## Description

Harpocrates is a POC providing a Permanent Fully Homomorphic Encryption (FHE) smart contract on Arweave.

This is done using [Nucypher](https://github.com/nucypher/)'s/[SunscreenTech](https://github.com/Sunscreen-tech/Sunscreen) sunscreen compiler, an easy to use FHE compiler that currently only uses the [BFV scheme](https://inferati.com/blog/fhe-schemes-bfv) (Brakerski-Fan-Vercauteren).
The backend is the permanent decentralized data storage open to all [Arweave](https://arweave.org).
The ZK circuits are built using [Circom](https://github.com/iden3/circom) and compiled using [`snarkjs`](https://github.com/iden3/snarkjs).
And the connection to a private key store is done using [WalletConnect](https://github.com/WalletConnect)

## Why should I care?

Fully Homomorphic Encryption allows you to keep your data private, which was never possible before. To be able to get analytics on your data, you would usually be required to decrypt it from you database, send it to a third party so they can crunch the numbers and get it back to you. This means that to gain insight into your own data, you have to give it **all** away. With FHE, you don't have to, these same insights can be gotten while never ever revealing the inputs (and even the output!).

This allows you to do multi-party computation, identity management, private database lookups, games with full privacy and more!

By mixing the FHE and decentralized open systems, we have a system that can't be stopped AND cannot be snooped on!

## ☢️☢️☢️⛔ ⛔ ⛔ ⚠️⚠️⚠️Warning⚠️⚠️⚠️⛔ ⛔ ⛔ ☢️☢️☢️

This repo is a POC, not meant to be in production. Use at your own risks.

## Motivation

**Privacy is a human right.**

Technology can be a double edge sword. It has made us extremely productive and inventive. It connects us like nothing else before. But this convenience and power can be used nefariously. We tend to forget that not so long ago, it was easy to be private. You would pay in cash, GPS didn't exist, cellphones even less so. All these technologies degrade our **option** to be private. You don't have to be private all the time, but you should have the option to at anytime. It's time to harness technology to swing the pendulum back the other way.

I strongly identify with statement from Eric Hughes:

> "We must defend our own privacy if we expect to have any. We must come together and create systems which allow anonymous transactions to take place. People have been defending their own privacy for centuries with whispers, darkness, envelopes, closed doors, secret handshakes, and couriers. The technologies of the past did not allow for strong privacy, but electronic technologies do."

> > Eric Hughes: A Cypherpunk's Manifesto.

Now we have the tools (or at least we are very nearly there) to have the **optionality** to be private. Let's make sure we get there!

This project is not perfect, but I hope it will have an impact on people and spark their imagination on other similar privacy preserving projects.

## How to build and run it

Make sure you have [`snarkjs`](https://github.com/iden3/snarkjs) installed.

To run this, first run `make` in the root of the project. This download the necessary dependencies and also set you up to be able to create and verify Zkproofs :).

Now, if you want this to be deployed to Arweave, you will need to have an arweave keyfile with some AR on it. You can get a pre-funded one [here](https://faucet.arweave.net/). Make sure to have the keyfile in the `fhe_contract` repository with the name `arweave-keyfile.json`.

Now all you have to do is go into the fhe_contract repository `fhe_contract` and run `cargo run -- run-all`. From there, just follow the necessary prompts!

More info can be found by running `cargo run -- --help`

```bash
POC providing a Permanent Fully Homomorphic Encryption smart contract on Arweave.

USAGE:
    fhe_contract [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    compute-latest     computes the result of all the FHE transactions
    create-new-user    creates the keys for a new user
    deploy             deploys the FHE contract to Arweave
    fetch-latest       fetches the latest transactions and saves them in the cache
    fetch-zk           fetches the latest zk params and saves it in the cache
    help               Print this message or the help of the given subcommand(s)
    init-state         initializes the state of our contract
    init-zk-proof      deploys all the information needed for ZKsnark to arweave
    run-all            runs all the interactions in the correct order, also is interactive
    vote               create and deploys a vote on the user's preferred proposition
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

- [x] Integrate WalletConnect

### Limitations

Here are the limitations encountered in the project

1. FHE is still very new. There aren't a lot of production grade libraries out there, and if they are they are extremely complex and easy to make a mistake. Thanks efforts like Nucypher's Sunscreen, it makes it easier for developers to use.

2. The current FHE scheme involved here cannot do (yet) an infinite amount of computation (at least using this library). The more computations happen, the more noise gets introduced into the ciphertext. When the noise gets too much, it is impossible to decrypt. There are techniques to go around this, but for now you are limited in the number of computations possible. Also, the current scheme doesn't allow for comparisons, which reduces the scope of possible actions.

3. In its current form in our application, the admin (the creator) of the vote can decrypt all of the votes. This is less than ideal, and could even be dangerous. One can kind of go around this by combining 2 or more (n) people's votes together and then deploying the txn to Arweave. There would have to be proofs that the n votes are valid.

4. The current way this project is done, the "voter apathy" problem is still not solved. This can be solved with Multi-key Fully Homomorphic Encryption (MKFHE) as we will discuss later.

5. The current scheme does not give a guarantee that what was passed as input to create the Zkproof is the same as what was encrypted. There is research that show a possibility for those proofs to [exists](https://eprint.iacr.org/2019/057.pdf) (in the BFV scheme and others!)

### Future possibilities

The advent of [Multi-key Fully Homomorphic Encryption](https://eprint.iacr.org/2021/1131.pdf) (MKFHE) can be a game changer. In MKFHE, every participant can encrypt their data using their key and then do calculations on this encrypted data that was created with different keys. There would then need to be a decryption phase of the result at the end, without ever needing to divulge the input to anyone else. This would remove the voter apathy, and not allow any of the users to know who voted for who.

The possibility of having proofs about the validity of the encrypted data as in [here](https://eprint.iacr.org/2019/057.pdf) will also be a game changer, as now one can be sure that the others are playing by the same rules.
