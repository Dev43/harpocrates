use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sunscreen::{Application, PrivateKey, PublicKey, Runtime};

use crate::calculator::get_initial_state;
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
    /// deploy
    CreateNewUser {},
    Deploy {},
    InitState {
        #[clap(value_parser)]
        contract_id: String,
    },
    FetchLatest {},
    Vote {
        #[clap(value_parser)]
        number: i8,
    },
    RunAll {},

    /// does testing things
    Test {
        /// lists test values
        #[clap(short, long, action)]
        list: bool,
    },
}

#[derive(Serialize, Deserialize)]
struct Keys {
    pub pk: String,
    pub sk: String,
}

fn write_to_file(name: String, data: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(format!("./.cache/{}", name))?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
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

            let (pk, _) = get_keys();

            // let res = decrypt(&contract_json, &pk, &sk);

            // println!("{:?}", res);

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
        Some(Commands::InitState { contract_id: cid }) => {
            let contract_json = compile().unwrap();

            let (pk, _) = get_keys();

            let contract_id = cid.clone();

            // let res = decrypt(&contract_json, &pk, &sk);

            // println!("{:?}", res);

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
        Some(Commands::FetchLatest {}) => {
            let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;
            ar.fetch_latest_state("28dygSSTZsbHVeOmEO69B0bS7aVzYWr2pFM1HCdosGg".to_string())
                .await
                .unwrap();
        }
        Some(Commands::Vote { number: e }) => {
            if e > &9 {
                println!("Invalid choice, only from 1-9");
                return Ok(());
            }

            // we need at least 1 other person to vote with us to somewhat obfuscate our vote. Hence, we will store the vote in the cache if
            // first to vote, otherwise we add up a vote with another person and publish it. (we can also do a peer to peer check to ensure it will vote as we want it to).
            // both parties will need to create a zkproof saying this was their vote (they participated in it).
            // this is mitigated if we use MKFHE - where everyone can encrypt their vote, publish it and have it all counted + decrypted at the end.

            println!("{}", e);
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

fn get_keys() -> (PublicKey, PrivateKey) {
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
