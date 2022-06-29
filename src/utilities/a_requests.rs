use std::time::Instant;

use anyhow::Result;
use floating_duration::TimeFormat;

#[derive(Debug)]
pub struct AsyncClientShim {
    pub client: reqwest::Client,
    pub auth_token: Option<String>,
    pub user_id: String,
    pub endpoint: String,
}

impl AsyncClientShim {
    pub fn new(endpoint: String, auth_token: Option<String>, user_id: String) -> AsyncClientShim {
        let client = reqwest::Client::new();

        AsyncClientShim {
            client,
            auth_token,
            user_id,
            endpoint,
        }
    }
}

pub async fn a_post<V>(client_shim: &AsyncClientShim, path: &str) -> Result<Option<V>>
where
    V: serde::de::DeserializeOwned,
{
    base_postb(client_shim, path, "{}").await
}

pub async fn a_postb<T, V>(client_shim: &AsyncClientShim, path: &str, body: T) -> Result<Option<V>>
where
    T: serde::ser::Serialize,
    V: serde::de::DeserializeOwned,
{
    base_postb(client_shim, path, body).await
}

pub async fn base_postb<T, V>(
    client_shim: &AsyncClientShim,
    path: &str,
    body: T,
) -> Result<Option<V>>
where
    T: serde::ser::Serialize,
    V: serde::de::DeserializeOwned,
{
    let start = Instant::now();

    let mut b = client_shim
        .client
        .post(&format!("{}/{}", client_shim.endpoint, path));

    if client_shim.auth_token.is_some() {
        b = b.bearer_auth(client_shim.auth_token.clone().unwrap());
        b = b.header("user_id", client_shim.user_id.clone());
    }
    let res = b.json(&body).send().await?;
    info!("(req {}, took: {})", path, TimeFormat(start.elapsed()));
    Ok(Some(res.json::<V>().await?))
}
