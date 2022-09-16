use sunscreen::{
    fhe_program,
    types::{bfv::Signed, Cipher},
};

#[fhe_program(scheme = "bfv")]
pub fn add_vote(
    curr_votes: [Cipher<Signed>; 10],
    vote: [Cipher<Signed>; 10],
) -> [Cipher<Signed>; 10] {
    let mut curr = curr_votes.clone();
    for i in 0..10 {
        curr[i] = curr[i] + vote[i];
    }

    curr
}

#[cfg(test)]
mod tests {
    use super::*;
    use sunscreen::{Compiler, Error, Runtime};

    #[test]
    fn it_works() -> Result<(), Error> {
        let app = Compiler::new().fhe_program(add_vote).compile()?;

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

        let first_result = runtime.run(
            app.get_program(add_vote).unwrap(),
            vec![init_state, alice_vote],
            &counter_pk,
        )?;

        let c: [Signed; 10] = runtime.decrypt(&first_result[0], &counter_sk)?;
        // intermediate result
        println!("{:?}", c);

        // now bob votes
        let bob_vote = runtime.encrypt(
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

        let fr = first_result[0].clone();

        let final_result = runtime.run(
            app.get_program(add_vote).unwrap(),
            vec![fr, bob_vote],
            &counter_pk,
        )?;

        let final_tally: [Signed; 10] = runtime.decrypt(&final_result[0], &counter_sk)?;
        println!("{:?}", final_tally);

        assert_eq!(final_tally[0].to_string(), "2".to_string());
        Ok(())
    }
}
