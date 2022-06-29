// #![no_main]

#[macro_use]
extern crate clap;
use clap::App;

use client::escrow;
use client::utilities::requests::ClientShim;
use client::wallet::{self, WALLET_FILENAME};
use floating_duration::TimeFormat;
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings"))
        .unwrap()
        .merge(config::Environment::new())
        .unwrap();
    let hm = settings.try_into::<HashMap<String, String>>().unwrap();
    let endpoint = hm.get("endpoint").unwrap();

    let mut client_shim = ClientShim::new(
        endpoint.to_string(),
        Some("cli_token".to_owned()),
        "cli_app".to_owned(),
    );

    // For async tests
    // let mut a_client_shim = AsyncClientShim::new(
    //     endpoint.to_string(),
    //     Some("cli_token".to_owned()),
    //     "cli_app".to_owned(),
    // );

    let network = "testnet".to_string();
    if let Some(matches) = matches.subcommand_matches("create-wallet") {
        println!("Network: [{}], Creating wallet", network);
        let coin_type: &str = matches.value_of("coin-type").unwrap();

        let coin_list = vec!["btc", "eth"];
        if !coin_list.contains(&coin_type) {
            panic!("Invalid coin type");
        }
        let token: &str = matches.value_of("token").unwrap();
        client_shim.auth_token = Some(token.to_owned());
        let wallet = wallet::Wallet::new(&client_shim, &network, coin_type);
        wallet.save();
        println!("Network: [{}], Wallet saved to disk", &network);

        println!("Network: [{}], Escrow initiated", &network);
    } else if let Some(matches) = matches.subcommand_matches("wallet") {
        let mut wallet: wallet::Wallet = wallet::Wallet::load();

        if matches.is_present("new-address") {
            wallet.get_crypto_address();
            wallet.save();
        } else if matches.is_present("get-balance") {
            wallet.get_balance();
        } else if matches.is_present("backup") {
            println!("Backup private share pending (it can take some time)...");
            let start = Instant::now();
            wallet.backup();

            println!(
                "Backup key saved in escrow (Took: {})",
                TimeFormat(start.elapsed())
            );
        } else if matches.is_present("verify") {
            let escrow = escrow::Escrow::load();

            println!("verify encrypted backup (it can take some time)...");

            let start = Instant::now();
            wallet.verify_backup(escrow);

            println!(" (Took: {})", TimeFormat(start.elapsed()));
        } else if matches.is_present("restore") {
            let escrow = escrow::Escrow::load();

            println!("backup recovery in process 📲 (it can take some time)...");

            let start = Instant::now();
            wallet::Wallet::recover_and_save_share(escrow, &network, &client_shim);

            println!(
                " Backup recovered 💾(Took: {})",
                TimeFormat(start.elapsed())
            );
        } else if matches.is_present("rotate") {
            println!("Rotating secret shares");

            let start = Instant::now();
            let token: &str = matches.value_of("token").unwrap();
            client_shim.auth_token = Some(token.to_owned());
            wallet.rotate(&client_shim, WALLET_FILENAME);

            println!(
                "key rotation complete, (Took: {})",
                TimeFormat(start.elapsed())
            );
        } else if matches.is_present("send") {
            if let Some(matches) = matches.subcommand_matches("send") {
                let from: &str = matches.value_of("from").unwrap();
                let to: &str = matches.value_of("to").unwrap();
                let amount: &str = matches.value_of("amount").unwrap();
                let token: &str = matches.value_of("token").unwrap();
                client_shim.auth_token = Some(token.to_owned());

                // a_client_shim.auth_token = Some(token.to_owned());

                wallet.send(
                    from,
                    to,
                    amount.to_string().parse::<f64>().unwrap(),
                    &client_shim,
                );

                if wallet.coin_type == "btc" {
                    wallet.save();
                }
            }
        }
    }
}
