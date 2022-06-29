use serde_json::json;

use crate::{dto::ecdsa::PrivateShare, utilities::requests::ClientShim};
use std::{collections::HashMap, fs};

pub const RINKEBY_TEST_API: &str =
    "wss://eth-rinkeby.alchemyapi.io/v2/UmSDyVix3dL4CtIxC2zlKkSuk2UoRw1J";
pub const PRIVATE_SHARE_FILENAME: &str = "test-assets/private_share.json";

#[derive(Debug, Deserialize)]
#[allow(dead_code, non_snake_case)]
struct AuthToken {
    StatusCode: u16,
    Msg: String,
}

pub struct MockToken {
    pub token: String,
    pub user_id: String,
}

pub fn get_test_private_share(filename: &str) -> PrivateShare {
    let data = fs::read_to_string(filename).expect("Unable to load test private_share!");
    serde_json::from_str(&data).unwrap()
}

fn mock_sign_in(email: &str, password: &str, signin_url: &str) -> MockToken {
    let http_client = reqwest::blocking::Client::new();
    let auth_body = json!({
        "email": email,
        "password": password
    });
    let http_resp = http_client
        .post(signin_url)
        .json(&auth_body)
        .send()
        .unwrap()
        .json::<AuthToken>()
        .unwrap();

    println!("{:#?}", http_resp);

    MockToken {
        token: http_resp.Msg,
        user_id: email.to_owned(),
    }
}

pub fn mock_client_shim(test_email: &str, test_pw: &str) -> ClientShim {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings"))
        .unwrap()
        .merge(config::Environment::new())
        .unwrap();
    let hm = settings.try_into::<HashMap<String, String>>().unwrap();
    let endpoint = hm.get("endpoint").unwrap();
    let email = hm.get(test_email).unwrap();
    let password = hm.get(test_pw).unwrap();
    let signin_url = hm.get("TEST_SIGNIN_URL").unwrap();

    let mock_token_obj = mock_sign_in(email, password, signin_url);

    ClientShim::new(
        endpoint.to_string(),
        Some(mock_token_obj.token),
        mock_token_obj.user_id,
    )
}

pub fn print_balance(balance: usize) {
    println!("Balance to test {:?}", balance);
}

pub fn print_tx_hash(hash: &str) {
    println!("Transaction hash: {:?}", hash);
}
