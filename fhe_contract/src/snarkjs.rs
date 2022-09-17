use std::process::Command;

pub fn verify_snark_proof() -> Result<String, Box<dyn std::error::Error>> {
    let mut snarkjs = Command::new("sh");
    snarkjs
        .arg("-c")
        .arg("snarkjs groth16 verify ./circom/verification_key.json ./circom/public.json ./circom/proof.json ");
    let hello_1 = snarkjs.output()?;

    Ok(String::from_utf8(hello_1.stdout).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it() {
        let _ = verify_snark_proof().unwrap();
    }
}
