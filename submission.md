# Web3Infra Submission

This repo is part of my submission to the Web3Infra hackathon by EverVision. It was originally created during EthBerlin and has been extended.

## What was done

First and foremost, I ported the essential features that I needed form [arseeding](https://github.com/everFinance/arseeding) and [everpay-go](https://github.com/everFinance/everpay-go) into rust. This repo can be found [here](https://github.com/Dev43/arseeding-rust).

This was then integrated into harpocrates (this repo) (an application to homomorphically vote on the arweave chain), speeding up the deployment and voting process by more than 10x.

## Deeper overview

The arseeding-rust library is close to feature parity. I unfortunately don't have the time/resources to finish it before the end of the hackathon. Also, there was work done to make the library WASM friendly, which meant changing [arloader](https://github.com/CalebEverett/arloader) WASM friendly too. This means that a developer should be able to create an application in rust, and deploy it on the browser.

The new library was then integrated into harpocrates, which was a very easy when all said and done. This gives the ability to pay with all the tokens available on everpay, for the deployment of the necessary data on chain.
