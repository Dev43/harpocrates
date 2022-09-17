use arloader::{
    error::Error,
    status::StatusCode,
    transaction::{Base64, FromUtf8Strs, Tag},
    Arweave,
};
// use futures::{stream, StreamExt};
use reqwest;
use ring::digest::{Context, SHA256};
use serde_json::{json, Value};
use std::str::FromStr;
use std::time::Duration;
use std::time::SystemTime;
use std::{path::PathBuf, time::UNIX_EPOCH};
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

        let action = r#"{"action":"deploy", arguments: [], validity_proof:"ID_OF_VALIDITY_PROOF"}"#;

        let tags = self.create_tags(&contract_id, &unix_timestamp, action, true);

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

        Ok((tx.id.to_string(), contract_id))
    }

    // fn create_and_send_tx() {}

    fn create_tags(
        &self,
        contract_id: &str,
        unix_timestamp: &str,
        action: &str,
        source: bool,
    ) -> Vec<Tag<Base64>> {
        let app = get_app_name(source);
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
            // Tag::<Base64>::from_utf8_strs(
            //     "Eth-Address",
            //     "0x...",
            // ).unwrap(),
            // Tag::<Base64>::from_utf8_strs(
            //     "Eth-Signature",
            //     "0x...",
            // )
            // .unwrap(),
        ]
    }

    pub async fn initialize_state(
        &self,
        contract_id: &str,
        initial_state: String,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let unix_timestamp = get_unix_timestamp();

        let action =
            r#"{"action":"init_state", arguments: [], validity_proof:"ID_OF_VALIDITY_PROOF"}"#;
        let tags = self.create_tags(&contract_id, &unix_timestamp, action, false);

        let mut tx = self
            .client
            .create_transaction(
                initial_state.as_bytes().to_vec(),
                Some(tags),
                None,
                // (60000000, 0) minimum price term for it to go through
                (60000000, 60000000),
                false,
            )
            .await
            .unwrap();

        tx = self.client.sign_transaction(tx).unwrap();

        println!("Init: Arweave Tx ID: {} ", tx.id);

        let res = self.client.post_transaction(&tx).await?;
        println!("{:?}", res);

        Ok((tx.id.to_string(), contract_id.to_string()))
    }

    pub async fn vote(
        &self,
        contract_id: &str,
        vote_data: String,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let unix_timestamp = get_unix_timestamp();

        // todo, currently we don't send out multiple votes, so no arguments of last votes
        let action = r#"{"action":"vote", arguments: [], validity_proof:"ID_OF_VALIDITY_PROOF"}"#;
        let tags = self.create_tags(&contract_id, &unix_timestamp, action, false);

        let mut tx = self
            .client
            .create_transaction(
                vote_data.as_bytes().to_vec(),
                Some(tags),
                None,
                // (60000000, 0) minimum price term for it to go through
                (60000000, 60000000),
                false,
            )
            .await
            .unwrap();

        tx = self.client.sign_transaction(tx).unwrap();

        println!("Vote: Arweave Tx ID: {} ", tx.id);

        let res = self.client.post_transaction(&tx).await?;
        println!("{:?}", res);

        Ok((tx.id.to_string(), contract_id.to_string()))
    }

    pub async fn fetch_latest_state(
        &self,
        contract_id: String,
    ) -> Result<(Vec<Value>, Vec<Value>), Error> {
        let source = graphql_query(&contract_id, true).await.unwrap();

        let interactions = graphql_query(&contract_id, false).await.unwrap();

        fs::write(
            "./.cache/transactions.json",
            json!({"source": source, "interactions":interactions }).to_string(),
        )
        .await
        .unwrap();

        Ok((source, interactions))
    }

    pub async fn wait_till_mined(&self, tx_id: &str) -> Result<(), Error> {
        let id = Base64::from_str(&tx_id).unwrap();

        let mut status = self.client.get_status(&id).await.unwrap();

        while status.status != StatusCode::Confirmed {
            tokio::time::sleep(Duration::from_secs(5)).await;
            status = self.client.get_status(&id).await.unwrap();
        }
        println!("Transaction with id {}, has been mined ", id);
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

fn get_app_name(source: bool) -> String {
    let mut app = "harpocrates-interactions";
    if source {
        app = "harpocrates-source";
    }
    app.to_string()
}

async fn graphql_query(contract_address: &str, source: bool) -> Result<Vec<Value>, Error> {
    let app = get_app_name(source);
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
        println!("{:?}", values.len());
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_waits_till_mined() -> Result<(), Box<dyn std::error::Error>> {
        let ar = Ar::new("./arweave-keyfile.json".to_string()).await;
        let res = ar
            .wait_till_mined("vPxIKj-kq7l1lXhVwJpNDIa1Xsz2lHR3TnpUDAHM4aQ")
            .await;
        println!("{:?}", res);
        Ok(())
    }
}
