use crate::contract::add_vote;
use sunscreen::{
    types::bfv::Signed, Application, Ciphertext, Error, FheProgramInput, PrivateKey, PublicKey,
    Runtime,
};
// todo add the inputs
pub fn calculate<I>(
    contract_json: String,
    pk: PublicKey,
    arguments: Vec<I>,
) -> Result<Ciphertext, Error>
where
    I: Into<FheProgramInput>,
{
    let app: Application = serde_json::from_str(&contract_json).unwrap();

    let runtime = Runtime::new(app.params())?;

    let final_result = runtime.run(app.get_program(add_vote).unwrap(), arguments, &pk)?;

    Ok(final_result[0].clone())
}

pub fn get_initial_state(contract_json: &str, pk: PublicKey) -> Result<String, Error> {
    let app: Application = serde_json::from_str(&contract_json).unwrap();

    let runtime = Runtime::new(app.params())?;

    let init_state = runtime.encrypt(
        [
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
        ],
        &pk,
    )?;

    let ser_json = serde_json::to_string(&init_state).unwrap();
    // println!("{}", ser_json);
    Ok(ser_json)
}

pub fn decrypt(contract_json: &str, pk: PublicKey, sk: PrivateKey) -> Result<String, Error> {
    let app: Application = serde_json::from_str(&contract_json).unwrap();

    let runtime = Runtime::new(app.params())?;

    let init_state = runtime.encrypt(
        [
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
        ],
        &pk,
    )?;

    let ser_json = serde_json::to_string(&init_state).unwrap();

    let c: [Signed; 10] = runtime.decrypt(&init_state, &sk)?;
    // intermediate result
    println!("{:?}", c);
    // println!("{}", ser_json);
    Ok(ser_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sunscreen::types::bfv::Signed;

    #[test]
    fn it_works() -> Result<(), Error> {
        let contract_json = std::fs::read_to_string("./compiled_contract/add_vote_params.json")
            .expect("Should have been able to read the file");

        let app: Application = serde_json::from_str(&contract_json).unwrap();

        let runtime = Runtime::new(app.params())?;

        let (counter_pk, counter_sk) = runtime.generate_keys()?;

        let init_state = runtime.encrypt(
            [
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
            ],
            &counter_pk,
        )?;

        let alice_vote = runtime.encrypt(
            [
                Signed::from(1),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
                Signed::from(0),
            ],
            &counter_pk,
        )?;

        let res = calculate(contract_json, counter_pk, vec![init_state, alice_vote])?;

        let final_tally: [Signed; 10] = runtime.decrypt(&res, &counter_sk)?;
        println!("{:?}", final_tally);
        assert_eq!(final_tally[0].to_string(), "1".to_string());

        Ok(())
    }

    #[test]
    fn it_get_init_state() -> Result<(), Error> {
        let contract_json = std::fs::read_to_string("./compiled_contract/add_vote_params.json")
            .expect("Should have been able to read the file");

        let app: Application = serde_json::from_str(&contract_json).unwrap();

        let runtime = Runtime::new(app.params())?;

        let (counter_pk, _) = runtime.generate_keys()?;

        let res = get_initial_state(&contract_json, counter_pk);

        Ok(())
    }
}
