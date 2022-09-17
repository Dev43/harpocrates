use std::process::Command;

pub fn verify_snark_proof(
    public_path: &str,
    proof_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut snarkjs = Command::new("sh");
    snarkjs.arg("-c").arg(format!(
        "snarkjs groth16 verify ./.cache/verification_key.json {} {} ",
        public_path, proof_path
    ));
    let output = snarkjs.output()?;

    let err = output.stderr;
    if err.len() != 0 {
        panic!("{}", String::from_utf8(err).unwrap())
    }
    Ok(String::from_utf8(output.stdout).unwrap())
}

pub fn generate_witness() -> Result<String, Box<dyn std::error::Error>> {
    let mut snarkjs = Command::new("sh");
    snarkjs
        .arg("-c")
        .arg("./bin/generate_witness/generate_witness ./.cache/input.json ./.cache/witness.wtns");
    let output = snarkjs.output()?;

    let err = output.stderr;
    if err.len() != 0 {
        panic!("{}", String::from_utf8(err).unwrap())
    }
    Ok(String::from_utf8(output.stdout).unwrap())
}

pub fn generate_proof() -> Result<String, Box<dyn std::error::Error>> {
    let mut snarkjs = Command::new("sh");
    snarkjs
        .arg("-c")
        .arg("snarkjs groth16 prove ./.cache/vote_is_valid_0001.zkey ./.cache/witness.wtns ./.cache/proof.json ./.cache/public.json");
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
        let r = verify_snark_proof("./.cache/input.json", "./.cache/proof.json")?;
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
