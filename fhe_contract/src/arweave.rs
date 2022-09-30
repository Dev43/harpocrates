use crate::ethereum::EthClient;
use arloader::{
    error::Error,
    status::StatusCode,
    transaction::{Base64, FromUtf8Strs, Tag},
    Arweave,
};
use arseeding_rust::{arseeding_types::ASError, client::ASClient};
use async_trait::async_trait;
use reqwest;
use ring::digest::{Context, SHA256};
use serde_json::{json, Value};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::{collections::HashMap, fmt::Write};
use std::{path::PathBuf, time::UNIX_EPOCH};
use tokio::fs;
use url::Url;

#[async_trait]
pub trait Uploader {
    async fn upload(&self, data: Vec<u8>, tags: Vec<Tag<Base64>>) -> Result<String, ASError>;
}
pub struct ArseedingUploader {
    client: ASClient,
}

impl ArseedingUploader {
    fn new(client: ASClient) -> Self {
        ArseedingUploader { client }
    }
}

#[async_trait]
impl Uploader for ArseedingUploader {
    async fn upload(&self, data: Vec<u8>, tags: Vec<Tag<Base64>>) -> Result<String, ASError> {
        let mut t = HashMap::new();
        for tag in tags {
            t.insert(
                tag.name.to_utf8_string().unwrap(),
                tag.value.to_utf8_string().unwrap(),
            );
        }

        let bundle_id = self.client.send_and_pay("AR", &t, data, "").await?;

        Ok(bundle_id)
    }
}

pub struct ArweaveUploader {
    client: Arweave,
}

impl ArweaveUploader {
    fn new(client: Arweave) -> Self {
        ArweaveUploader { client }
    }
}

#[async_trait]
impl Uploader for ArweaveUploader {
    async fn upload(&self, data: Vec<u8>, tags: Vec<Tag<Base64>>) -> Result<String, ASError> {
        let mut tx = self
            .client
            .create_transaction(data, Some(tags), None, (60000000, 60000000), false)
            .await
            .unwrap();

        tx = self.client.sign_transaction(tx).unwrap();

        let _res = self.client.post_transaction(&tx).await?;

        Ok(tx.id.to_string())
    }
}

#[derive(PartialEq, Copy, Clone)]
enum ContractType {
    Source,
    Interaction,
    ZkSnark,
}

pub struct Ar {
    client: Arweave,
    uploader: Arc<dyn Uploader>,
}

impl Ar {
    pub async fn new(path: String) -> Self {
        let arweave = Arweave::from_keypair_path(
            PathBuf::from(path.clone()),
            Url::from_str("http://arweave.net").unwrap(),
        )
        .await
        .unwrap();
        let arweave2 = Arweave::from_keypair_path(
            PathBuf::from(path),
            Url::from_str("http://arweave.net").unwrap(),
        )
        .await
        .unwrap();

        let uploader = Arc::new(ArweaveUploader::new(arweave2));

        Ar {
            client: arweave,
            uploader: uploader,
        }
    }

    // {
    //     "App-Name": "harpocrates",
    //     "App-Version": "0.0.1",
    //     "Contract": "0x...",
    //     "Content-Type": "application/json",
    //     "Sunscreen-Version": "0.0.1",
    //     "Validity-Proof": "ZkSnark/circom@2.0.8/snarkjs@0.4.27",
    //     "Unix-Time": "134546456456",
    //     "Input": "{}"
    // }
    // Returns Arweave TXID and contract TXID
    pub async fn deploy_contract(
        &self,
        contract_data: &str,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let unix_timestamp = get_unix_timestamp();
        // contract id is only the hash of the data and a unix timestamp -- this is only a POC - not secure as someone can change the unix timestamp
        let mut id_data = contract_data.clone().to_string();
        id_data.push_str(&unix_timestamp);

        let contract_id = Base64(sha_256(id_data.as_bytes()).to_vec()).to_string();

        let action = r#"{"action":"deploy", arguments: []}"#;

        let (account, sig) = get_eth_metadata(&contract_data.as_bytes().to_vec()).await?;

        let tags = create_tags(
            &contract_id,
            &unix_timestamp,
            action,
            ContractType::Source,
            &account,
            &sig,
        );

        let tx = self
            .uploader
            .upload(contract_data.as_bytes().to_vec(), tags)
            .await
            .unwrap();

        Ok((tx, contract_id))
    }

    pub async fn deploy_zksnark(
        &self,
        contract_id: &str,
        contract_data: Vec<u8>,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let unix_timestamp = get_unix_timestamp();
        // contract id is only the hash of the data and a unix timestamp -- this is only a POC - not secure as someone can change the unix timestamp

        let action = r#"{"action":"zk_snark", arguments: []}"#;

        let (account, sig) = get_eth_metadata(&contract_data).await?;

        let tags = create_tags(
            &contract_id,
            &unix_timestamp,
            action,
            ContractType::ZkSnark,
            &account,
            &sig,
        );

        let tx = self.uploader.upload(contract_data, tags).await.unwrap();

        Ok((tx, contract_id.to_string()))
    }

    pub async fn initialize_state(
        &self,
        contract_id: &str,
        initial_state: String,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let unix_timestamp = get_unix_timestamp();

        let action = r#"{"action":"init_state", arguments: []}"#;
        let (account, sig) = get_eth_metadata(&initial_state.as_bytes().to_vec()).await?;

        let tags = create_tags(
            &contract_id,
            &unix_timestamp,
            action,
            ContractType::Interaction,
            &account,
            &sig,
        );

        let tx = self
            .uploader
            .upload(initial_state.as_bytes().to_vec(), tags)
            .await
            .unwrap();

        Ok((tx.to_string(), contract_id.to_string()))
    }

    pub async fn vote(
        &self,
        contract_id: &str,
        vote_data: String,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let unix_timestamp = get_unix_timestamp();

        // todo, currently we don't send out multiple votes, so no arguments of last votes
        let action = r#"{"action":"vote", arguments: []}"#;
        let (account, sig) = get_eth_metadata(&vote_data.as_bytes().to_vec()).await?;

        let tags = create_tags(
            &contract_id,
            &unix_timestamp,
            action,
            ContractType::Interaction,
            &account,
            &sig,
        );

        let tx = self
            .uploader
            .upload(vote_data.as_bytes().to_vec(), tags)
            .await
            .unwrap();

        Ok((tx.to_string(), contract_id.to_string()))
    }

    pub async fn fetch_latest_state(
        &self,
        contract_id: String,
    ) -> Result<(Vec<Value>, Vec<Value>), Error> {
        let source = graphql_query(&contract_id, ContractType::Source)
            .await
            .unwrap();

        let interactions = graphql_query(&contract_id, ContractType::Interaction)
            .await
            .unwrap();

        fs::write(
            "./.cache/transactions.json",
            json!({"source": source, "interactions":interactions}).to_string(),
        )
        .await
        .unwrap();

        Ok((source, interactions))
    }
    pub async fn fetch_zk(&self, contract_id: String) -> Result<Vec<u8>, Error> {
        let zk_snark = zk_query(&contract_id, ContractType::ZkSnark).await.unwrap();

        fs::write("./.cache/zksnark.bin", &zk_snark).await.unwrap();

        Ok(zk_snark)
    }

    pub async fn wait_till_mined(&self, tx_id: &str) -> Result<(), Error> {
        let id = Base64::from_str(&tx_id).unwrap();

        let mut status = self.client.get_status(&id).await.unwrap();

        while status.status != StatusCode::Confirmed {
            tokio::time::sleep(Duration::from_secs(5)).await;
            status = self.client.get_status(&id).await.unwrap();
        }
        Ok(())
    }
}

fn get_unix_timestamp() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs().to_string()
}

pub fn sha_256(data: &[u8]) -> [u8; 32] {
    let mut ctx2 = Context::new(&SHA256);
    ctx2.update(data);
    ctx2.finish().as_ref().try_into().expect("incorrect length")
}

const QUERY: &str = r#"query Interactions ($app: String!, $block_min: Int, $contract_address: String!, $follow_cursor: String) {
    transactions(
      sort: HEIGHT_ASC
      after: $follow_cursor
      tags: [
        { name: "App-Name", values: [$app] }
        { name: "Contract", values: [$contract_address] }
      ]
      block: {min: $block_min, max: 1000000000}
    ) {
      edges {
        cursor  
        node {
          id
          block {
            timestamp
          }
          owner {
            address
            key
          }
  
          tags {
            name
            value
          }
        }
      }
    }
  }"#;

fn get_record(value: &Value) -> Value {
    let obj = value.as_object().unwrap();
    json!({
      "cursor": obj["cursor"],
      "id": obj["node"]["id"],
      "owner": obj["node"]["owner"]["address"],
      "tags": obj["node"]["tags"],
    })
}

fn get_app_name(contract_type: ContractType) -> String {
    let app = match contract_type {
        ContractType::Source => "harpocrates-source",
        ContractType::Interaction => "harpocrates-interactions",
        ContractType::ZkSnark => "harpocrates-zksnark",
    };

    app.to_string()
}

async fn graphql_query(
    contract_address: &str,
    contract_type: ContractType,
) -> Result<Vec<Value>, Error> {
    let mut values = fetch(contract_address, contract_type).await.unwrap();

    for v in values.iter_mut() {
        let resp = reqwest::get(format!(
            "https://arweave.net/{}/data.json",
            v["id"].clone().as_str().unwrap()
        ))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
        v.as_object_mut().unwrap().insert("data".to_string(), resp);
    }

    Ok(values)
}

async fn fetch(contract_address: &str, contract_type: ContractType) -> Result<Vec<Value>, Error> {
    let app = get_app_name(contract_type);
    let mut values: Vec<Value> = Vec::new();
    let resp = reqwest::Client::new()
        .post("https://arweave.net/graphql")
        .json(&json!({ "query": QUERY, "operationName": "Interactions", "variables": json!({"app": app, "block_min":1, "contract_address":contract_address,"follow_cursor": ""})}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let mut transactions: Vec<Value> = resp.as_object().unwrap()["data"]["transactions"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(get_record)
        .collect();

    while transactions.len() > 0 {
        values.append(&mut transactions);
        let cursor = values.last().unwrap().as_object().unwrap()["cursor"]
            .as_str()
            .unwrap();
        let resp = reqwest::Client::new()
            .post("https://arweave.net/graphql")
            .json(&json!({ "query": QUERY, "operationName": "Interactions", "variables": json!({"app": app, "block_min":1, "contract_address":contract_address, "follow_cursor": cursor})}))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();

        transactions = resp.as_object().unwrap()["data"]["transactions"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .map(get_record)
            .collect();
    }
    Ok(values)
}

async fn zk_query(contract_address: &str, contract_type: ContractType) -> Result<Vec<u8>, Error> {
    let values = fetch(contract_address, contract_type).await.unwrap();

    let resp = reqwest::get(format!(
        "https://arweave.net/{}/data.json",
        values[0]["id"].clone().as_str().unwrap()
    ))
    .await
    .unwrap()
    .bytes()
    .await
    .unwrap();

    return Ok(resp.to_vec());
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_waits_till_mined() -> Result<(), Box<dyn std::error::Error>> {
        let ar = Ar::new("./arweave-keyfile.json".to_string()).await;
        let _res = ar
            .wait_till_mined("vPxIKj-kq7l1lXhVwJpNDIa1Xsz2lHR3TnpUDAHM4aQ")
            .await;
        Ok(())
    }

    #[tokio::test]
    async fn graphql_query_test() -> Result<(), Box<dyn std::error::Error>> {
        let _res = zk_query(
            "28dygSSTZsbHVeOmEO69B0bS7aVzYWr2pFM1HCdosGg",
            ContractType::ZkSnark,
        )
        .await
        .unwrap();
        Ok(())
    }
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

pub async fn get_eth_metadata(
    data: &Vec<u8>,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // to show you own your ethereum address, you need to sign a message
    // the message is the concatenation of your address + hash of the data being deployed

    // we get the hash
    let hash = sha_256(&data);

    let c = EthClient::new().await?;

    let account = c.account();
    let account_b = account.as_bytes();

    let mut iter = account_b.into_iter().chain(&hash);
    let result = [(); 74].map(|_| iter.next().unwrap().to_owned());

    let hashed_message = encode_hex(&sha_256(&result));

    c.get_sig(&hashed_message).await
}

fn create_tags(
    contract_id: &str,
    unix_timestamp: &str,
    action: &str,
    contract_type: ContractType,
    eth_address: &str,
    eth_sig: &str,
) -> Vec<Tag<Base64>> {
    let app = get_app_name(contract_type);
    vec![
        Tag::<Base64>::from_utf8_strs("App-Name", &app).unwrap(),
        Tag::<Base64>::from_utf8_strs("App-Version", "0.0.1").unwrap(),
        Tag::<Base64>::from_utf8_strs("Contract", &contract_id).unwrap(),
        Tag::<Base64>::from_utf8_strs("Content-Type", "application/json").unwrap(),
        Tag::<Base64>::from_utf8_strs("Sunscreen-Version", "0.6.1").unwrap(),
        Tag::<Base64>::from_utf8_strs("Validity-Proof", "ZkSnark/circom@2.0.8/snarkjs@0.4.27")
            .unwrap(),
        Tag::<Base64>::from_utf8_strs("Unix-Time", &unix_timestamp).unwrap(),
        Tag::<Base64>::from_utf8_strs("Input", &action).unwrap(),
        Tag::<Base64>::from_utf8_strs("Eth-Address", eth_address).unwrap(),
        Tag::<Base64>::from_utf8_strs("Eth-Signature", eth_sig).unwrap(),
    ]
}
