use std::process::Command;

pub fn verify_snark_proof() -> Result<String, Box<dyn std::error::Error>> {
    let mut snarkjs = Command::new("sh");
    snarkjs
        .arg("-c")
        .arg("snarkjs groth16 verify ./circom/verification_key.json ./circom/public.json ./circom/proof.json ");
    let output = snarkjs.output()?;

    let err = output.stderr;
    if err.len() != 0 {
        panic!("{}", String::from_utf8(err).unwrap())
    }
    Ok(String::from_utf8(output.stdout).unwrap())
}

pub fn generate_witness() -> Result<String, Box<dyn std::error::Error>> {
    let mut snarkjs = Command::new("sh");
    snarkjs.arg("-c").arg(
        "./bin/generate_witness ./circom/vote_is_valid_js/vote_is_valid.wasm ./circom/input.json witness.wtns",
    );
    let output = snarkjs.output()?;

    let err = output.stderr;
    if err.len() != 0 {
        panic!("{}", String::from_utf8(err).unwrap())
    }
    Ok(String::from_utf8(output.stdout).unwrap())
}

// as part of proof
// public.json
// proof.json

pub fn generate_proof() -> Result<String, Box<dyn std::error::Error>> {
    let mut snarkjs = Command::new("sh");
    snarkjs
        .arg("-c")
        .arg("snarkjs groth16 prove ./circom/vote_is_valid_0001.zkey ./circom/witness.wtns ./circom/proof.json ./circom/public.json");
    let output = snarkjs.output()?;

    let err = output.stderr;
    if err.len() != 0 {
        panic!("{}", String::from_utf8(err).unwrap())
    }
    Ok(String::from_utf8(output.stdout).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it() -> Result<(), Box<dyn std::error::Error>> {
        let r = verify_snark_proof()?;
        println!("{}", r);
        Ok(())
    }

    #[test]
    fn test_it_2() -> Result<(), Box<dyn std::error::Error>> {
        let r = generate_witness()?;
        println!("{}", r);
        Ok(())
    }

    #[test]
    fn test_it_3() -> Result<(), Box<dyn std::error::Error>> {
        let r = generate_proof()?;
        println!("{}", r);
        Ok(())
    }
}
