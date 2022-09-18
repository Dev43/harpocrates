use std::error::Error;
use walletconnect::{qr, Client, Metadata};

pub struct EthClient {
    client: Client,
    account: String,
}

impl EthClient {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let c = Client::new(
            "ethberlin",
            Metadata {
                description: "Ethberlin WallectConnect for harpocrates".into(),
                url: "https://github.com/nlordell/walletconnect-rs".parse()?,
                icons: vec!["https://avatars0.githubusercontent.com/u/4210206".parse()?],
                name: "EthBerlin WallectConnect for harpocrates".into(),
            },
        )?;

        let (accounts, _) = c.ensure_session(qr::print).await?;

        let main_account = format!("{:?}", accounts[0]);
        Ok(EthClient {
            client: c,
            account: main_account,
        })
    }

    pub fn account(&self) -> String {
        self.account.clone()
    }
    pub async fn get_sig(&self, to_sign: &str) -> Result<(String, String), Box<dyn Error>> {
        let sig = self.client.personal_sign(&[to_sign, &self.account]).await?;
        Ok((self.account.to_string(), format!("{}", sig)))
    }
}

#[allow(unused)]
fn verify_sig(message: &str, sig: &[u8]) -> Result<String, Box<dyn Error>> {
    let addr = walletconnect::client::verify_sig(message, sig)?;
    Ok(addr)
}

#[cfg(test)]
mod tests {

    use walletconnect::ethers_core::utils::{hex, keccak256};

    use super::*;

    #[tokio::test]
    async fn it_runs() -> Result<(), Box<dyn std::error::Error>> {
        let msg = "hello";
        let h = keccak256(msg.as_bytes());
        let message_hash = hex::encode(&h);

        let c = EthClient::new().await.unwrap();
        let (acc, sig) = c.get_sig(msg).await.unwrap();
        println!("{}", acc);
        println!("{}", sig);

        verify_sig(&message_hash, sig.as_bytes())?;
        Ok(())
    }
}
