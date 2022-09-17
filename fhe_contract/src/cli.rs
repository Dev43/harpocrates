use clap::{Parser, Subcommand};

use crate::compiler::compile;
use serde_json::json;
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
        Some(Commands::Deploy {}) => {
            let data = compile().unwrap();

            let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

            let res = ar.deploy_contract(data).await?;

            println!("{} {}", res.0, res.1);
            write_to_file(
                "deployment.json".to_string(),
                json!({"arweave_id": res.0, "contract_id": res.1}).to_string(),
            )?;
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