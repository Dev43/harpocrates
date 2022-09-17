use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sunscreen::{Application, PublicKey, Runtime};

use crate::calculator::{decrypt, get_initial_state};
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
    FetchLatest {},
    PublishAction {
        #[clap(value_parser)]
        action: String,
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
            let app: Application = serde_json::from_str(&contract_json).unwrap();

            let runtime = Runtime::new(app.params()).unwrap();

            let pk_string = std::fs::read_to_string("./.cache/pk.json")
                .expect("Should have been able to read the file");

            let pk: PublicKey = serde_json::from_str(&pk_string).unwrap();

            let raw_keys = std::fs::read_to_string("./.cache/sk.json")
                .expect("Should have been able to read the file");
            let keys: Value = serde_json::from_str(&raw_keys).unwrap();
            let secret_k: Vec<u8> = serde_json::from_value(keys["sk"].clone()).unwrap();
            println!("{:?}", secret_k);

            let sk = runtime.bytes_to_private_key(&secret_k).unwrap();

            println!("{:?}", sk);

            let res = decrypt(&contract_json, pk, sk);

            // let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

            // let res = ar.deploy_contract(&contract_json).await?;

            // println!("{} {}", res.0, res.1);
            // write_to_file(
            //     "deployment.json".to_string(),
            //     json!({"arweave_id": res.0, "contract_id": res.1}).to_string(),
            // )?;

            // // get the init state, all vectors of 0
            // get_initial_state(&contract_json, pk);

            // ar.init_state(contract_id, initial_state)
        }
        Some(Commands::FetchLatest {}) => {
            let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;
            let res = ar
                .fetch_latest_state("qSBgFlQaZhI0Uv785pRQMiJkAMVDkdmZvSH4PoMXPZM".to_string())
                .await
                .unwrap();

            println!("{:?}", res);
        }
        Some(Commands::PublishAction { action: e }) => {
            println!("{}", e);
        }
        Some(Commands::RunAll {}) => {}
        None => {}
    }

    // let pb = indicatif::ProgressBar::new(100);
    // for i in 0..100 {
    //     pb.inc(1);
    // }
    // pb.finish_with_message("done");

    Ok(())
}
