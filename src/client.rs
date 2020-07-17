use hyper::StatusCode;
use reqwest;
use reqwest::{Body, Error, Response};

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone)]
struct ResponseId(String);

struct Client {
    server_url: String,
    api_key: String,
}

impl Client {
    fn new(server_url: &str, api_key: &str) -> Client {
        Client {
            server_url: server_url.trim_end_matches("/").to_string(),
            api_key: api_key.to_string(),
        }
    }

    async fn get_response(&self, response_id: ResponseId) -> Result<Response, Error> {
        reqwest::Client::builder()
            .build()?
            .get(&format!(
                "{url}/aliens/{response_id}",
                url = self.server_url,
                response_id = response_id.0
            ))
            .query(&[("apiKey", self.api_key.clone())])
            .send()
            .await
    }

    async fn send<T: Into<String>>(&self, content: T) -> Result<Response, Error> {
        reqwest::Client::builder()
            .build()?
            .post(&format!("{url}/aliens/send", url = self.server_url))
            .body(Body::from(content.into()))
            .query(&[("apiKey", self.api_key.clone())])
            .send()
            .await
    }

    async fn echo<T: Into<String>>(&self, content: T) -> Result<Response, Error> {
        reqwest::Client::builder()
            .build()?
            .post(&format!("{url}", url = self.server_url))
            .body(Body::from(content.into()))
            .query(&[("apiKey", self.api_key.clone())])
            .send()
            .await
    }
}

#[cfg(test)]
mod tests;
