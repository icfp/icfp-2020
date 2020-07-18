use crate::ast::{modulate_to_string, Symbol};
use crate::client::Client;
use std::fs::read_to_string;

#[tokio::test]
async fn try_send() {
    let api_key = read_to_string("api_key").unwrap();
    let client = Client::new("https://icfpc2020-api.testkontur.ru/", &api_key);
    let response = client.send(api_key).await.unwrap();
    dbg!(response);
}

#[tokio::test]
async fn send_list() {
    let api_key = read_to_string("api_key").unwrap();
    let client = Client::new("https://icfpc2020-api.testkontur.ru/", &api_key);

    use Symbol::*;

    let symbol = Pair(Lit(1).into(), Nil.into());

    let response = client.send(modulate_to_string(&symbol)).await.unwrap();
    dbg!(dbg!(response).text().await.unwrap());

    let symbol = Pair(Lit(2).into(), Pair(Lit(54214).into(), Nil.into()).into());

    let response = client
        .send(dbg!(modulate_to_string(&symbol)))
        .await
        .unwrap();
    dbg!(dbg!(response).text().await.unwrap());
}
