use reqwest::Client;

pub struct Authenticator {
    pub client: Client,
    pub key: String,
}

#[tokio::main]
async fn main() {
    println!("")
    // test_macro()
}
//
// #[tokio::main]
// async fn main() {
//     get_summoner_names(
//         Client::builder()
//             .danger_accept_invalid_certs(true)
//             .build()
//             .unwrap(),
//     )
//     .await
//     .unwrap();
// }
//
// async fn get_summoner_names(client: Client) -> anyhow::Result<Vec<String>> {
//     let resp = client
//         .get("https://127.0.0.1:2999/liveclientdata/allgamedata")
//         .send()
//         .await?;
//     let val: serde_json::Value = resp.json().await?;
//     for player in val.get("allPlayers").unwrap().as_array().unwrap() {
//         print!(
//             "{}-euw,",
//             player.get("summonerName").unwrap().as_str().unwrap()
//         );
//     }
//     Ok(vec![])
// }
