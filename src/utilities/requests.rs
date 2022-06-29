use anyhow::{anyhow, Result};
use floating_duration::TimeFormat;
use serde;
use std::time::Instant;

#[derive(Debug)]
pub struct ClientShim {
    pub client: reqwest::blocking::Client,
    pub auth_token: Option<String>,
    pub user_id: String,
    pub endpoint: String,
}

impl ClientShim {
    pub fn new(endpoint: String, auth_token: Option<String>, user_id: String) -> ClientShim {
        let client = reqwest::blocking::Client::builder().build().unwrap();

        ClientShim {
            client,
            auth_token,
            user_id,
            endpoint,
        }
    }
}

pub fn post<V>(client_shim: &ClientShim, path: &str) -> Result<Option<V>>
where
    V: serde::de::DeserializeOwned,
{
    _postb(client_shim, path, "{}")
}

pub fn postb<T, V>(client_shim: &ClientShim, path: &str, body: T) -> Result<Option<V>>
where
    T: serde::ser::Serialize,
    V: serde::de::DeserializeOwned,
{
    _postb(client_shim, path, body)
}

fn _postb<T, V>(client_shim: &ClientShim, path: &str, body: T) -> Result<Option<V>>
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
    let res = b.json(&body).send();

    info!("(req {}, took: {})", path, TimeFormat(start.elapsed()));

    let value = match res {
        Ok(v) => v.text()?,
        Err(e) => return Err(anyhow!("HTTP POST with auth token failed: {}", e)),
    };

    Ok(Some(serde_json::from_str(value.as_str())?))
}
