#[cfg(debug_assertions)]
use smol_db_client::SmolDbClient;
#[cfg(debug_assertions)]
use smol_db_common::db_packets::db_settings::DBSettings;
#[cfg(debug_assertions)]
use std::time::Instant;

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

        {
            let start = Instant::now();
            let _ = client.create_db("test_db123", DBSettings::default());
            let end = Instant::now();
            println!("{}", end.duration_since(start).as_micros());
        }

        {
            let start = Instant::now();
            let _ = client.create_db("test_db1234", DBSettings::default());
            let end = Instant::now();
            println!("{}", end.duration_since(start).as_micros());
        }

        {
            let start = Instant::now();
            let _ = client.create_db("test_db12345", DBSettings::default());
            let end = Instant::now();
            println!("{}", end.duration_since(start).as_micros());
        }

        {
            let db_name = "test_db123";
            {
                let start = Instant::now();
                let _ = client.write_db(db_name, "location1", "lmao1");
                let end = Instant::now();
                println!("{}", end.duration_since(start).as_micros());
            }

            {
                let start = Instant::now();
                let _ = client.write_db(db_name, "location2", "lmao2");
                let end = Instant::now();
                println!("{}", end.duration_since(start).as_micros());
            }
            {
                let start = Instant::now();
                let _ = client.write_db(db_name, "location3", "lmao2");
                let end = Instant::now();
                println!("{}", end.duration_since(start).as_micros());
            }
            {
                let start = Instant::now();
                let _ = client.write_db(db_name, "location4", "lmao3");
                let end = Instant::now();
                println!("{}", end.duration_since(start).as_micros());
            }
        }

        {
            let db_name = "test_db1234";

            {
                let start = Instant::now();
                let _ = client.write_db(db_name, "location1", "lmao11");
                let end = Instant::now();
                println!("{}", end.duration_since(start).as_micros());
            }

            {
                let start = Instant::now();
                let _ = client.write_db(db_name, "location2", "lmao22");
                let end = Instant::now();
                println!("{}", end.duration_since(start).as_micros());
            }

            {
                let start = Instant::now();
                let _ = client.write_db(db_name, "location3", "lmao23");
                let end = Instant::now();
                println!("{}", end.duration_since(start).as_micros());
            }

            {
                let start = Instant::now();
                let _ = client.write_db(db_name, "location4", "lmao33");
                let end = Instant::now();
                println!("{}", end.duration_since(start).as_micros());
            }
        }
        let role = client.get_role("test_db123").unwrap();

        println!("{:?}", role);
    }
}
