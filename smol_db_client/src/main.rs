#[cfg(debug_assertions)]
use smol_db_client::SmolDbClient;
use smol_db_common::prelude::DBSettings;

fn main() {
    #[cfg(not(debug_assertions))]
    {
        let crate_name: &str = env!("CARGO_PKG_NAME");
        let crate_version: &str = env!("CARGO_PKG_VERSION");
        panic!("This crate: {} version: {} has an executable, but is not meant to be run, and it seems that it has been run, I hope you enjoy this message though!", crate_name, crate_version);
    }

    #[cfg(debug_assertions)]
    {
        let key = "test_key_123";
        let mut client = SmolDbClient::new("localhost:8222").unwrap();
        client.set_access_key(key.to_string()).unwrap();

        client.setup_encryption();

        let resp = client.create_db("blah",DBSettings::default());
        println!("{:?}", resp);



    }
}
