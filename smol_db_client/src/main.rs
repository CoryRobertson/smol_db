#![allow(unused_variables, dead_code,unused_imports)] // TODO: remove this lints

use smol_db_client::Client;
use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_response::DBPacketResponse;
use smol_db_common::db_packets::db_settings::DBSettings;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::time::{Duration, Instant};

fn main() {
    let key = "test_key_123";
    let mut client = Client::new("localhost:8222").unwrap();
    client.set_access_key(key.to_string()).unwrap();
    client.create_db("test_db123", DBSettings::default());
    client.create_db("test_db1234", DBSettings::default());
    client.create_db("test_db12345", DBSettings::default());

    let role = client.get_role("test_db123").unwrap();

    println!("{:?}", role);
}
