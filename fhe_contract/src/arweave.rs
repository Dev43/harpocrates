use arloader::{
    error::Error,
    transaction::{Base64, FromUtf8Strs, Tag},
    Arweave,
};
use ring::digest::{Context, SHA256};
use std::{path::PathBuf, time::UNIX_EPOCH};

use reqwest;
use serde_json::{json, Value};
use std::str::FromStr;
use std::time::SystemTime;
use tokio::fs;
use url::Url;

pub struct Ar {
    client: Arweave,
}

impl Ar {
    pub async fn new(path: String) -> Self {
        let arweave = Arweave::from_keypair_path(
            PathBuf::from(path),
            Url::from_str("http://arweave.net").unwrap(),
        )
        .await
        .unwrap();

        Ar { client: arweave }
    }

    // Returns Arweave TXID and contract TXID
    pub async fn deploy_contract(
        &self,
        contract_data: String,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let unix_timestamp = &since_the_epoch.as_secs().to_string();

        let mut id_data = contract_data.clone();
        id_data.push_str(unix_timestamp);

        let id = Base64(sha_256(id_data.as_bytes()).to_vec()).to_string();

        let tags = vec![
            Tag::<Base64>::from_utf8_strs("App-Name", "harpocrates").unwrap(),
            Tag::<Base64>::from_utf8_strs("App-Version", "0.0.1").unwrap(),
            Tag::<Base64>::from_utf8_strs("Contract", &id).unwrap(),
            Tag::<Base64>::from_utf8_strs("Content-Type", "application/json").unwrap(),
            Tag::<Base64>::from_utf8_strs("Sunscreen-Version", "0.6.1").unwrap(),
            Tag::<Base64>::from_utf8_strs("Validity-Proof", "ZkSnark/circom@2.0.8/snarkjs@0.4.27")
                .unwrap(),
            Tag::<Base64>::from_utf8_strs("Unix-Time", unix_timestamp).unwrap(),
            Tag::<Base64>::from_utf8_strs(
                "Input",
                r#"{"action":"deploy", arguments: [], validity_proof:"ID_OF_VALIDITY_PROOF"}"#,
            )
            .unwrap(),
        ];

        let mut tx = self
            .client
            .create_transaction(
                contract_data.as_bytes().to_vec(),
                Some(tags),
                None,
                // (60000000, 0) minimum price term for it to go through
                (60000000, 0),
                false,
            )
            .await
            .unwrap();

        tx = self.client.sign_transaction(tx).unwrap();

        let res = self.client.post_transaction(&tx).await?;
        println!("{:?}", res);

        println!("Arweave Tx ID: {} ", tx.id);
        println!("Contract inner ID: {} ", id);

        Ok((tx.id.to_string(), id))
    }

    pub async fn fetch_latest_state(&self, contract_id: String) -> Result<Vec<Value>, Error> {
        graphql_query(contract_id).await
    }
}

pub fn sha_256(data: &[u8]) -> [u8; 32] {
    let mut ctx2 = Context::new(&SHA256);
    ctx2.update(data);
    ctx2.finish().as_ref().try_into().expect("incorrect length")
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
const QUERY: &str = r#"query Interactions ($block_min: Int, $contract_address: String!, $follow_cursor: String) {
    transactions(
      sort: HEIGHT_ASC
      after: $follow_cursor
      tags: [
        { name: "App-Name", values: ["harpocrates"] }
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

async fn graphql_query(contract_address: String) -> Result<Vec<Value>, Error> {
    let mut values: Vec<Value> = Vec::new();
    let resp = reqwest::Client::new()
        .post("https://arweave.net/graphql")
        .json(&json!({ "query": QUERY, "operationName": "Interactions", "variables": json!({"block_min":1, "contract_address":contract_address,"follow_cursor": ""})}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    println!("{:#?}", resp);

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
            .json(&json!({ "query": QUERY, "operationName": "Interactions", "variables": json!({"block_min":1, "contract_address":contract_address, "follow_cursor": cursor})}))
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
        println!("{:?}", values.len());
    }
    fs::write("data.json", serde_json::to_string(&values).unwrap())
        .await
        .unwrap();

    Ok(values)
}
