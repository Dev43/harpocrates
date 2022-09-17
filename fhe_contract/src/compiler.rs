use crate::contract::add_vote;
use std::io::prelude::*;
use std::{error::Error, fs::File};
use sunscreen::{Compiler, FheProgramFn};
pub fn compile_and_save_contract() -> Result<(), Box<dyn Error>> {
    let ser_app_json = compile().unwrap();
    // we output it to a target folder
    let mut file = File::create(format!("./.cache/{}_params.json", add_vote.name()))?;
    file.write_all(ser_app_json.as_bytes())?;

    // reserializes the app
    // let new_app: Application = serde_json::from_str(&ser_app_json).unwrap();

    // // Serializes the encrypted values
    // let ser = bincode::serialize(&a).unwrap();
    // let ser_json = serde_json::to_string(&a).unwrap();
    Ok(())
}

pub fn compile() -> Result<String, Box<dyn Error>> {
    // first we compile the app
    let app = Compiler::new().fhe_program(add_vote).compile().unwrap();

    // we then serialize it to json (can be bincode too, json for clarity)
    let ser_app_json = serde_json::to_string(&app).unwrap();

    Ok(ser_app_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(), Box<dyn Error>> {
        compile_and_save_contract()
    }
}
