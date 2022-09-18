use clap::{Parser, Subcommand};
use prettytable::{row, Cell, Row, Table};
use serde::{Deserialize, Serialize};
use sunscreen::types::bfv::Signed;
use sunscreen::{Application, Ciphertext, PrivateKey, PublicKey, Runtime};

use crate::ascii;
use crate::calculator::{calculate, decrypt, get_initial_state};
use crate::compiler::compile;
use crate::snarkjs::{generate_proof, generate_witness, verify_snark_proof};
#[allow(unused)]
use owo_colors::OwoColorize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::prelude::*;
use std::os::unix::prelude::PermissionsExt;
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

#[derive(Serialize, Deserialize)]
struct ZKProof {
    pub proof: String,
    pub public: String,
}

#[derive(Serialize, Deserialize)]
struct VoteData {
    pub data: String,
    pub zkp: ZKProof,
}

fn write_to_file(name: String, data: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(format!("./.cache/{}", name))?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

fn create_new_user() -> Result<(), Box<dyn std::error::Error>> {
    let contract_json = compile().unwrap();

    let app: Application = serde_json::from_str(&contract_json).unwrap();

    let runtime = Runtime::new(app.params()).unwrap();

    let (pk, sk) = runtime.generate_keys().unwrap();

    #[allow(unused)]
    fs::create_dir("./.cache");

    write_to_file("pk.json".to_string(), (serde_json::to_string(&pk)).unwrap()).unwrap();
    write_to_file("sk.json".to_string(), json!({ "sk": sk }).to_string()).unwrap();
    Ok(())
}

async fn deploy() -> Result<String, Box<dyn std::error::Error>> {
    let contract_json = compile().unwrap();

    let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

    let res = ar.deploy_contract(&contract_json).await?;
    let contract_id = res.1;
    let tx_id = res.0;

    write_to_file(
        "deployment.json".to_string(),
        json!({"arweave_id": tx_id, "contract_id": contract_id}).to_string(),
    )?;

    println!("Deploy: Arweave Tx ID: {} ", tx_id);

    // we wait till mined (main txn for now)
    let _mined_res = ar.wait_till_mined(&tx_id).await.unwrap();

    println!("Deploy: Contract inner ID: {} ", contract_id);

    Ok(contract_id)
}

async fn init_zk(id: &String) -> Result<(), Box<dyn std::error::Error>> {
    let contract_id = id.clone();

    let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

    let verification_key = read_file("./circom/verification_key.json").unwrap();
    let vote_is_valid_0001_zkey = read_file("./circom/vote_is_valid_0001.zkey").unwrap();
    let generate_witness = read_file("./bin/generate_witness/generate_witness").unwrap();

    let zk = ZkInfo {
        verification_key: verification_key,
        vote_is_valid_0001_zkey: vote_is_valid_0001_zkey,
        generate_witness: generate_witness,
    };

    let zk_data = bincode::serialize(&zk).unwrap();

    let res = ar.deploy_zksnark(&contract_id, zk_data).await?;
    let tx_id = res.0;

    println!("ZKSnark: Arweave Tx ID: {} ", tx_id);

    // we wait till mined (main txn for now)
    let _mined_res = ar.wait_till_mined(&tx_id).await.unwrap();

    println!(
        "ZKSnark: ZKSnark initialized\n Arweave Tx ID: {} \n For Contract ID: {}",
        tx_id, contract_id
    );
    Ok(())
}
async fn init_state(cid: &String) -> Result<(), Box<dyn std::error::Error>> {
    let contract_json = compile().unwrap();

    let (pk, _) = get_main_keys();

    let contract_id = cid.clone();

    let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

    // get the init state, all vectors of 0
    let init_state = get_initial_state(&contract_json, &pk).unwrap();

    let r = ar.initialize_state(&contract_id, init_state).await.unwrap();

    // we wait till mined (main txn for now)
    let _mined_res = ar.wait_till_mined(&r.0).await.unwrap();

    println!(
        "Init: State for Contract ID {} has been initialized ",
        contract_id
    );
    Ok(())
}
async fn fetch_latest(cid: &String) -> Result<(), Box<dyn std::error::Error>> {
    let contract_id = cid.clone();

    let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;
    ar.fetch_latest_state(contract_id.to_string())
        .await
        .unwrap();
    println!("Successfully fetched transactions, it is located at .cache/transactions.json");
    Ok(())
}
async fn fetch_zk(cid: &String) -> Result<(), Box<dyn std::error::Error>> {
    let contract_id = cid.clone();

    let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;
    let zk_data = ar.fetch_zk(contract_id.to_string()).await.unwrap();

    let all: ZkInfo = bincode::deserialize(&zk_data).unwrap();

    let mut file = File::create(format!("./.cache/{}", "generate_witness".to_string()))?;
    let metadata = file.metadata()?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o777);
    file.write_all(&all.generate_witness)?;
    let mut file = File::create(format!(
        "./.cache/{}",
        "vote_is_valid_0001.zkey".to_string()
    ))?;
    file.write_all(&all.vote_is_valid_0001_zkey)?;
    let mut file = File::create(format!("./.cache/{}", "verification_key.json".to_string()))?;
    file.write_all(&all.verification_key)?;

    println!("Successfully fetched Zk information, it is located at .cache/zksnark.bin");
    Ok(())
}
async fn compute_latest() -> Result<(), Box<dyn std::error::Error>> {
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
        // we verify the zkp

        // first deserialize the inputs
        // need to do this because of some weird bug with serde

        /*
        thread 'main' panicked at 'called `Result::unwrap()` on an
        `Err` value: Error("invalid type: string \"params\", expected a borrowed string", line: 0, column: 0)', /

        happens when serde_json::from_value(intx["data"].clone()).unwrap();
        */
        // todo add verifying eth sig
        // this bit does the calculations
        #[allow(unused_assignments)]
        let mut t_s = String::from("");
        if !intx["data"].is_array() && intx["data"]["data"].is_string() {
            t_s = intx["data"]["data"].as_str().unwrap().to_string();
            let zkp = intx["data"]["zkp"].as_object().unwrap();

            let proof: String = serde_json::from_value(zkp["proof"].clone()).unwrap();
            let public: String = serde_json::from_value(zkp["public"].clone()).unwrap();

            write_to_file("proof_to_check.json".to_string(), proof)?;
            write_to_file("public_input_to_check.json".to_string(), public)?;

            println!(
                "Verifying ZKSnark for {}",
                intx["id"].clone().as_str().unwrap()
            );
            match verify_snark_proof(
                "./.cache/public_input_to_check.json",
                "./.cache/proof_to_check.json",
            ) {
                Ok(_) => {}
                Err(_) => {
                    // if the ZKsnark is not valid, we skip this txn

                    println!(">>>>>>>>>>>>> Warning <<<<<<<<<<<<<");
                    println!(">>>>>>>>>>>>> Warning <<<<<<<<<<<<<");
                    println!(">>>>>>>>>>>>> Warning <<<<<<<<<<<<<");
                    println!(">>>>>>>>>>>>> ZKSnark not valid, skipping this txn <<<<<<<<<<<<<");
                    println!(">>>>>>>>>>>>> Warning <<<<<<<<<<<<<");
                    println!(">>>>>>>>>>>>> Warning <<<<<<<<<<<<<");
                    println!(">>>>>>>>>>>>> Warning <<<<<<<<<<<<<");
                    continue;
                }
            };

            println!("ZKProof verified {}", intx["id"].clone().as_str().unwrap());

            // we take out the proof.json and the input.json, save it and run the verify proof on them
        } else {
            t_s = serde_json::to_string(&intx["data"]).unwrap();
        }

        let input: Ciphertext = serde_json::from_str(&t_s).unwrap();

        let args = vec![curr_calc, input.clone()];
        curr_calc = calculate(&app, &pk, args).unwrap();
    }

    let decrypted = decrypt(&app, curr_calc, &sk).unwrap();

    // then we decrypt the output calculation
    println!("Compute Latest: current poll is {:?}", decrypted);

    let mut table = Table::new();
    table.add_row(row!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]);
    let rows = [
        decrypted[0],
        decrypted[1],
        decrypted[2],
        decrypted[3],
        decrypted[4],
        decrypted[5],
        decrypted[6],
        decrypted[7],
        decrypted[8],
        decrypted[9],
    ];

    let normal_rows: [Cell; 10] = rows.map(|x| Cell::new(&x.to_string()));

    table.add_row(Row::new(normal_rows.to_vec()));

    // we clear the screen
    clear_screen();
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());
    // Print the table to stdout
    table.printstd();
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());
    println!("{}", ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>".green());

    Ok(())
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

async fn vote(id: &String, index: &usize) -> Result<(), Box<dyn std::error::Error>> {
    if index > &9 && index < &0 {
        println!("Invalid choice, only from 0-9");
        return Ok(());
    }

    let contract_id = id.clone();

    let contract_json = compile().unwrap();

    let app: Application = serde_json::from_str(&contract_json).unwrap();

    let runtime = Runtime::new(app.params()).unwrap();

    let ar = crate::arweave::Ar::new("./arweave-keyfile.json".to_string()).await;

    let (pk, _) = get_main_keys();

    // we need at least 1 other person to vote with us to somewhat obfuscate our vote. Hence, we will store the vote in the cache if
    // first to vote, otherwise we add up a vote with another person and publish it. (we can also do a peer to peer check to ensure it will vote as we want it to).
    // both parties will need to create a zkproof saying this was their vote (they participated in it).
    // this is mitigated if we use MKFHE - where everyone can encrypt their vote, publish it and have it all counted + decrypted at the end.

    // this is where MKFHE would come in, some schemes (bfv etc show research) can show a validity proof of the encryption (as in, I can show you my vote is valid)
    let mut og_vote = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    og_vote[*index] = 1;

    // let's create a proof that our vote is valid

    // we create a file called input.json that will be the input to our circuit (our vote)
    write_to_file(
        "input.json".to_string(),
        json!({ "vote": og_vote }).to_string(),
    )?;

    // we now create a witness
    generate_witness()?;

    // we now generate the proof
    generate_proof()?;

    let proof_string = std::fs::read_to_string("./.cache/proof.json")?;

    let public_string = std::fs::read_to_string("./.cache/public.json")?;

    let zkp = ZKProof {
        proof: proof_string,
        public: public_string,
    };

    let vote = og_vote.map(|x| Signed::from(x));

    // we encrypt it
    let vote_enc = runtime.encrypt(vote, &pk).unwrap();

    let v_d = serde_json::to_string(&vote_enc).unwrap();

    let vote_data = VoteData {
        data: v_d,
        zkp: zkp,
    };

    let vote_data_string = serde_json::to_string(&vote_data).unwrap();

    // wait for it to get mined
    let res = ar.vote(&contract_id, vote_data_string).await.unwrap();
    println!("Vote: Your vote is being sent ArID {} ", res.0);

    // we wait till mined (main txn for now)
    let _mined_res = ar.wait_till_mined(&res.0).await.unwrap();

    println!("Vote: Your vote has been mined for {} ", contract_id);
    Ok(())
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("{}", ascii::HARPOCRATES);

    let _ = match &cli.command {
        Some(Commands::CreateNewUser {}) => create_new_user(),
        Some(Commands::Deploy {}) => {
            deploy().await?;
            Ok(())
        }
        Some(Commands::InitZkProof { contract_id: id }) => Ok(init_zk(id).await?),
        Some(Commands::InitState { contract_id: cid }) => Ok(init_state(cid).await?),
        Some(Commands::FetchLatest { contract_id: cid }) => Ok(fetch_latest(cid).await?),
        Some(Commands::FetchZk { contract_id: cid }) => Ok(fetch_zk(cid).await?),
        Some(Commands::ComputeLatest {}) => Ok(compute_latest().await?),
        Some(Commands::Vote {
            contract_id: id,
            number: index,
        }) => Ok(vote(id, index).await?),
        Some(Commands::RunAll {}) => Ok(run_all().await?),
        None => Ok(()),
    };
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

// TODO make this interactive
async fn run_all() -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!();
    println!();
    println!();
    println!(
        "Welcome to Harpocrates, where you can create a trusted decentralized vote, without telling the world how you voted."
    );
    println!( "We do this by using Fully Homomorphic Encryption (FHE), ZKSnarks and a decentralized storage solution");
    println!( "Today we are going to go through a scenario where we need to vote between 10 different propositions");
    println!( "I, and I imagine you too, do not want everyone to know who I am voting for. This is where FHE is quite interesting.");
    println!("But let's get going first!");

    println!();
    println!();
    println!("We will start by creating a set of keys. These keys will be stored in a .cache folder in the fhe_contract directory.");
    println!("Most useful files will be stored in this folder, of course after we fetch them from the storage solution first");

    prompt_for_any("Press any key to Continue (Ctrl+C to stop)");

    clear_screen();
    println!();
    println!();
    println!();
    println!("Creating main user...");
    println!("(this main user is the one that administrates the vote)");
    // create a new user
    create_new_user()?;
    println!();
    println!();
    println!();
    println!("Main user created! Welcome whoever you are! You now have a set of keys to be able to encrypt/decrypt and do some awesome calculations on encrypted data  ");
    println!();
    println!();
    println!();
    println!("Now, I've already created an FHE friendly contract, that simply allows us to keep track of votes coming in for a specific propositions");
    println!("What we will want to do now, is compile it and deploy it to a decentralized storage (either permanent like arweave or not like IPFS)");

    prompt_for_any("Press any key to Continue (Ctrl+C to stop)");

    clear_screen();

    println!();
    println!();
    println!("Ok, we're now going to deploy this contract to Arweave. As long as you have some AR in your wallet, there shouldn't be any issue.");
    println!("What you will also need is an Ethereum wallet with a WalletConnect functionality (such as Metamask)");
    println!("The reason is simple, for every transaction that we do, we need to keep track of who is sending these, we don't necessarily want everyone to vote on this, but just a subset of people. ");
    println!("We also need to ensure that this person is actually who they say they are. And so, you will need to sign a piece of data before we deploy this. ");

    clear_screen();

    println!();
    println!();
    println!();
    println!("A QR code should be visible to you right now. Use any WalletConnect compatible wallet and scan the code");
    println!("It will first ask you to connect, and then ask you to sign a piece of message. Please do both");

    prompt_for_any("Press any key to Continue (Ctrl+C to stop)");

    println!();
    println!();
    println!();
    println!("Deploying... this can take sometime");

    // deploy contract to arweave
    // let contract_id = deploy().await?;

    clear_screen();

    println!();
    println!();
    println!();
    println!("Great! Now we've deployed the contract to Arweave. Now we will need to deploy all the necessary information for our ZKsnark circuit, so others can verify");
    println!();
    println!();
    println!();
    prompt_for_any("Press any key to Continue (Ctrl+C to stop)");
    clear_screen();

    println!();
    println!(
        "Deploying ZK Params... this will take some time. Again, you will need to sign a message"
    );
    // init_zk(&contract_id).await?;

    clear_screen();

    println!();
    println!();
    println!();
    println!("Next up, we need to initialize our contract state. We are using the event model (or lazy evaluation)");
    println!("which means that instead of having all the miners of a network validating a transaction in a smart contract");
    println!("only the people that care about this data would. You essentially download all the transactions for this contract, and evaluate the outcome");
    println!("locally. Our initial state is very simple, it's an array of ENCRYPTED numbers, all set to 0 (i.e [0,0,0,0,0,0,0,0,0,0])");
    println!();
    println!();
    println!();
    prompt_for_any("Press any key to Continue (Ctrl+C to stop)");

    println!();
    println!("Deploying initial state... this will take some time");

    // init_state(&contract_id).await?;

    clear_screen();

    println!();
    println!();
    println!();
    println!("Ok, now that we've deployed all of our needed things, we can start fetching them (as though we didn't create them!)");
    println!("We will fetch all of the ZK params that we deployed earlier, these will be necessary for us later on");
    println!();
    println!();
    prompt_for_any("Press any key to Continue (Ctrl+C to stop)");

    clear_screen();
    println!();
    println!("Fetching ZK state (kinda fast!)");

    // fetch the zk info to populate our cache
    // fetch_zk(&contract_id).await?;

    clear_screen();

    println!();
    println!();
    println!();
    println!("Sweet, now let's get to the good stuff and actually voting!");
    println!("We have deployed all that we need now, our contract is deployed, our state is bare and ready to get votes.");
    println!();
    prompt_for_any("Press any key to Continue (Ctrl+C to stop)");
    println!();
    println!();

    let index = prompt_for_int(
        "Choose your favorite proposition between 0 and 9 (10 total)",
        0,
        9,
        "out of range you silly goose!",
    );

    println!("Now let's explain a bit more what is happening with your vote.");
    println!("Essentially what is happening is that we are taking your input (for example 1), and creating an array");
    println!("that represents your vote (i.e. [0,1,0,0,0,0,0,0,0,0] - note the 2nd element is 1)");
    println!("but we can't just send that up on the blockchain, otherwise people will know who you voted for");
    println!("so what we do is we encrypt it using an FHE scheme. This encryption is done using the voter admin's public key");
    println!("now as this gets deployed on the blockchain, no one will know who you voted for (except the voter admin, which we will talk later about even him/her not knowing!)");
    println!("But how do we know that you voted correctly? How do we know that you didn't put 100 votes for your proposition?");
    println!("This is where the ZkSnarks come in. You see, in addition to your vote, there is also a ZKproof that proves that you voted correctly.");
    println!("Now it's not perfect, it's not a proof that shows directly that the encryption that was produced comes from a valid input");
    println!("but there are some FHE schemes (this one - bfv - actually has some research on this ) that are starting to have these proofs, which is super cool ");
    println!();
    println!("So to recap, we are sending an encrypted version of your vote and a ZKproof to prove that your vote should be valid ");

    // vote on who we want
    // vote(&contract_id, &index).await?;

    clear_screen();

    // fetch all the txn, the latest
    // fetch_latest(&contract_id).await?;

    clear_screen();

    // compute the current outcome
    // compute_latest().await?;

    Ok(())
}

fn prompt_for_any(prompt: &str) {
    // Blank the terminal
    println!();
    println!();
    println!("{}", prompt);

    loop {
        let mut line = String::default();
        std::io::stdin().read_line(&mut line).unwrap();
        return;
    }
}

fn prompt_for_int(prompt: &str, min: i64, max: i64, out_of_range_message: &str) -> i64 {
    let int_val;

    // Blank the terminal
    println!("{}", prompt);

    loop {
        let mut line = String::default();
        std::io::stdin().read_line(&mut line).unwrap();

        match line.trim().parse::<i64>() {
            Ok(v) => {
                if v > max || v < min {
                    println!("{}", out_of_range_message);
                } else {
                    int_val = v;
                    break;
                }
            }
            _ => {
                println!("Not an integer. Try again.");
            }
        };
    }

    int_val
}
