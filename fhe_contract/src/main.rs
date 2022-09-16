use fhe_contract::add_vote;
use std::io::prelude::*;
use std::{error::Error, fs::File};
use sunscreen::{Compiler, FheProgramFn};

// main exports all the needed information from the FHE contract so we can have it on chain
fn main() -> Result<(), Box<dyn Error>> {
    // first we compile the app
    let app = Compiler::new().fhe_program(add_vote).compile().unwrap();

    // we then serialize it to json (can be bincode too, json for clarity)
    let ser_app_json = serde_json::to_string(&app).unwrap();

    // we output it to a target folder
    let mut file = File::create(format!(
        "./compiled_contract/{}_params.json",
        add_vote.name()
    ))?;
    file.write_all(ser_app_json.as_bytes())?;

    // // Serializes the encrypted values
    // let ser = bincode::serialize(&a).unwrap();
    // let ser_json = serde_json::to_string(&a).unwrap();
    Ok(())
}
