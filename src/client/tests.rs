use crate::client::Client;
use std::fs::read_to_string;

#[tokio::test]
async fn try_echo() {
    let api_key = read_to_string("api_key").unwrap();
    let client = Client::new("https://icfpc2020-api.testkontur.ru/", &api_key);
    let response = client.echo("test").await.unwrap();
    dbg!(response);
}
