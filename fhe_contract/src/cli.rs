use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sunscreen::types::bfv::Signed;
use sunscreen::{Application, Ciphertext, PrivateKey, PublicKey, Runtime};

use crate::calculator::{calculate, decrypt, get_initial_state};
use crate::compiler::compile;
use serde_json::{json, Value};
use std::fs::File;
use std::io::prelude::*;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    CreateNewUser {},
    Deploy {},
    InitZkProof {
        #[clap(value_parser)]
        contract_id: String,
    },
    InitState {
        #[clap(value_parser)]
        contract_id: String,
    },
    FetchLatest {
        #[clap(value_parser)]
        contract_id: String,
    },
    FetchZk {
        #[clap(value_parser)]
        contract_id: String,
    },
    ComputeLatest {},
    Vote {
        #[clap(value_parser)]
        contract_id: String,

        #[clap(value_parser)]
        number: usize,
    },
    RunAll {},
}

#[derive(Serialize, Deserialize)]
struct Keys {
    pub pk: String,
    pub sk: String,
}

#[derive(Serialize, Deserialize)]
struct Transactions {
    pub interactions: Vec<Value>,
    pub source: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
struct ZkInfo {
    pub verification_key: Vec<u8>,
    pub vote_is_valid_0001_zkey: Vec<u8>,
    pub generate_witness: Vec<u8>,
}

fn write_to_file(name: String, data: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(format!("./.cache/{}", name))?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::CreateNewUser {}) => {
            let contract_json = compile().unwrap();

            let app: Application = serde_json::from_str(&contract_json).unwrap();

            let runtime = Runtime::new(app.params()).unwrap();

            let (pk, sk) = runtime.generate_keys().unwrap();

            write_to_file("pk.json".to_string(), (serde_json::to_string(&pk)).unwrap()).unwrap();
            write_to_file("sk.json".to_string(), json!({ "sk": sk }).to_string()).unwrap();
        }
        Some(Commands::Deploy {}) => {
            let contract_json = compile().unwrap();

            let (pk, _) = get_main_keys();

            let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

            let res = ar.deploy_contract(&contract_json).await?;
            let contract_id = res.1;
            let tx_id = res.0;

            write_to_file(
                "deployment.json".to_string(),
                json!({"arweave_id": tx_id, "contract_id": contract_id}).to_string(),
            )?;

            // we wait till mined (main txn for now)
            let mined_res = ar.wait_till_mined(&tx_id).await.unwrap();
            println!("{:?}", mined_res);

            println!("Deploy: Arweave Tx ID: {} ", tx_id);
            println!("Deploy: Contract inner ID: {} ", contract_id);

            // get the init state, all vectors of 0
            let init_state = get_initial_state(&contract_json, &pk).unwrap();

            let r = ar.initialize_state(&contract_id, init_state).await.unwrap();
            println!("{:?}", r);

            // we wait till mined (main txn for now)
            let mined_res = ar.wait_till_mined(&r.0).await.unwrap();
            println!("{:?}", mined_res);
        }
        Some(Commands::InitZkProof { contract_id: id }) => {
            let contract_id = id.clone();

            let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

            let verification_key = read_file("./circom/verification_key.json").unwrap();
            let vote_is_valid_0001_zkey = read_file("./circom/vote_is_valid_0001.zkey").unwrap();
            let generate_witness = read_file("./bin/generate_witness").unwrap();

            // TO DEPLOY - after the whole ceremony
            // verification_key.json
            // vote_is_valid_0001.zkey
            // generate_witness
            let zk = ZkInfo {
                verification_key: verification_key,
                vote_is_valid_0001_zkey: vote_is_valid_0001_zkey,
                generate_witness: generate_witness,
            };

            let zk_data = bincode::serialize(&zk).unwrap();

            // let all: ZkInfo = bincode::deserialize(&zk_data).unwrap();

            // let mut file = File::create(format!("./.cache/{}", "genny".to_string()))?;
            // file.write_all(&all.generate_witness)?;

            let res = ar.deploy_zksnark(&contract_id, zk_data).await?;
            let tx_id = res.0;

            println!("ZKSnark: Arweave Tx ID: {} ", tx_id);

            // we wait till mined (main txn for now)
            let mined_res = ar.wait_till_mined(&tx_id).await.unwrap();
            println!("{:?}", mined_res);

            println!(
                "ZKSnark: ZKSnark initialized\n Arweave Tx ID: {} \n For Contract ID: {}",
                tx_id, contract_id
            );
        }
        Some(Commands::InitState { contract_id: cid }) => {
            let contract_json = compile().unwrap();

            let (pk, _) = get_main_keys();

            let contract_id = cid.clone();

            let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

            // get the init state, all vectors of 0
            let init_state = get_initial_state(&contract_json, &pk).unwrap();

            let r = ar.initialize_state(&contract_id, init_state).await.unwrap();
            println!("{:?}", r);

            // we wait till mined (main txn for now)
            let mined_res = ar.wait_till_mined(&r.0).await.unwrap();
            println!("{:?}", mined_res);

            println!(
                "Init: State for Contract ID {} has been initialized ",
                contract_id
            );
        }
        Some(Commands::FetchLatest { contract_id: cid }) => {
            let contract_id = cid.clone();

            let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;
            ar.fetch_latest_state(contract_id.to_string())
                .await
                .unwrap();
            println!(
                "Successfully fetched transactions, it is located at .cache/transactions.json"
            );
        }
        Some(Commands::FetchZk { contract_id: cid }) => {
            let contract_id = cid.clone();

            let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;
            ar.fetch_zk(contract_id.to_string()).await.unwrap();

            println!("Successfully fetched Zk information, it is located at .cache/zksnark.bin");
        }
        Some(Commands::ComputeLatest {}) => {
            // we get the contract from source

            let (pk, sk) = get_main_keys();

            let txs_string = std::fs::read_to_string("./.cache/transactions.json")
                .expect("Should have been able to read the file");

            let txns: Transactions = serde_json::from_str(&txs_string).unwrap();

            let source = txns.source[0].clone();

            let app: Application = serde_json::from_value(source["data"].clone()).unwrap();

            let intxs = txns.interactions;

            // we get the init state first
            let init = &intxs[0];
            let t_s = serde_json::to_string(&init["data"]).unwrap();
            let mut curr_calc: Ciphertext = serde_json::from_str(&t_s).unwrap();

            // we go through all transactions and run them one by one through the compiled contract
            for intx in intxs {
                // first deserialize the inputs
                // need to do this because of some weird bug with serde

                /*
                thread 'main' panicked at 'called `Result::unwrap()` on an
                `Err` value: Error("invalid type: string \"params\", expected a borrowed string", line: 0, column: 0)', /

                happens when serde_json::from_value(intx["data"].clone()).unwrap();
                */

                let t_s = serde_json::to_string(&intx["data"]).unwrap();
                let input: Ciphertext = serde_json::from_str(&t_s).unwrap();

                let args = vec![curr_calc, input.clone()];
                curr_calc = calculate(&app, &pk, args).unwrap();
            }

            let decrypted = decrypt(&app, curr_calc, &sk).unwrap();

            // then we decrypt the output calculation
            println!("Compute Latest: current poll is {:?}", decrypted);
        }
        Some(Commands::Vote {
            contract_id: id,
            number: index,
        }) => {
            if index > &9 && index < &0 {
                println!("Invalid choice, only from 0-9");
                return Ok(());
            }

            let contract_id = id.clone();

            let contract_json = compile().unwrap();

            let app: Application = serde_json::from_str(&contract_json).unwrap();

            let runtime = Runtime::new(app.params()).unwrap();

            let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

            // TODO CURRENTLY WE DONT HAVE IDENTIFICATION OF THE USER -- need to do this with ETH/walletconnect + signature
            // println!("Creating brand new user");
            // let (new_user_pk, new_user_sk) = runtime.generate_keys().unwrap();
            // println!("Brand new user created");

            let (pk, _) = get_main_keys();

            // we need at least 1 other person to vote with us to somewhat obfuscate our vote. Hence, we will store the vote in the cache if
            // first to vote, otherwise we add up a vote with another person and publish it. (we can also do a peer to peer check to ensure it will vote as we want it to).
            // both parties will need to create a zkproof saying this was their vote (they participated in it).
            // this is mitigated if we use MKFHE - where everyone can encrypt their vote, publish it and have it all counted + decrypted at the end.

            // it will create a vote and send it to arweave
            let mut vote = [
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
            ];

            vote[*index] = Signed::from(1);

            // we encrypt it

            let vote_enc = runtime.encrypt(vote, &pk).unwrap();

            let vote_data = serde_json::to_string(&vote_enc).unwrap();
            // wait for it to get mined
            let res = ar.vote(&contract_id, vote_data).await.unwrap();

            // we wait till mined (main txn for now)
            let mined_res = ar.wait_till_mined(&res.0).await.unwrap();
            println!("{:?}", mined_res);

            println!("Vote: Your vote has been mined for {} ", contract_id);
        }
        Some(Commands::RunAll {}) => {}
        None => {}
    }

    // show a progress bar as we move along!
    // let pb = indicatif::ProgressBar::new(100);
    // for i in 0..100 {
    //     pb.inc(1);
    // }
    // pb.finish_with_message("done");

    Ok(())
}

fn get_main_keys() -> (PublicKey, PrivateKey) {
    let contract_json = compile().unwrap();
    let app: Application = serde_json::from_str(&contract_json).unwrap();

    let runtime = Runtime::new(app.params()).unwrap();

    let pk_string = std::fs::read_to_string("./.cache/pk.json")
        .expect("Should have been able to read the file");

    let pk: PublicKey = serde_json::from_str(&pk_string).unwrap();

    let raw_keys = std::fs::read_to_string("./.cache/sk.json")
        .expect("Should have been able to read the file");
    let keys: Value = serde_json::from_str(&raw_keys).unwrap();
    let secret_k: Vec<u8> = serde_json::from_value(keys["sk"].clone()).unwrap();

    let sk = runtime.bytes_to_private_key(&secret_k).unwrap();

    (pk, sk)
}

fn read_file(path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    return Ok(buf);
}
