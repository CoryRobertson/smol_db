#![allow(unused_variables, dead_code)] // TODO: remove this lints

use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_response::DBPacketResponse;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::time::Instant;

fn main() {
    let mut buf: [u8; 1024] = [0; 1024];

    let packet1 = DBPacket::new_create_db("test1");
    let packet2 = DBPacket::new_write("test1", "location1", "data12");
    let packet3 = DBPacket::new_read("test1", "location1");
    let packet4 = DBPacket::new_delete_db("test1");

    // let packs = vec![
    //     packet1.serialize_packet(),
    //     packet2.serialize_packet(),
    //     packet3.serialize_packet(),
    //     packet4.serialize_packet(),
    // ];

    let mut client = TcpStream::connect("localhost:8222").unwrap();

    // let pack_bytes = packet1.serialize_packet().unwrap();
    // let _ = client.write(pack_bytes.as_bytes());
    // let read_res = client.read(&mut buf);

    // let pack_bytes = packet2.serialize_packet().unwrap();
    // let _ = client.write(pack_bytes.as_bytes());
    // let read_res = client.read(&mut buf);
    for _ in 0..10 {
        let pack_bytes = packet3.serialize_packet().unwrap();
        let start = Instant::now();
        let _ = client.write(pack_bytes.as_bytes());
        let read_res = client.read(&mut buf);
        let end = Instant::now();
        match read_res {
            Ok(len) => {
                let pack_data = from_utf8(&buf[0..len]).unwrap_or_default();
                println!("ok: {:?}", pack_data);
                let response: DBPacketResponse<String> =
                    serde_json::from_slice(&buf[0..len]).unwrap();
                println!("{:?}", response);
                println!("rtt: {}", end.duration_since(start).as_micros());
            }
            Err(_) => {}
        }
    }

    // for pack_res in packs {
    //     // test a bunch of packet types just for testing.
    //     let pack = pack_res.unwrap();
    //     let pack_bytes = pack.as_bytes();
    //     let _ = client.write(pack_bytes);
    //     let read_res = client.read(&mut buf);
    //     match read_res {
    //         Ok(len) => {
    //             println!("ok: {:?}", from_utf8(&buf[0..len]).unwrap_or_default());
    //         }
    //         Err(_) => {
    //             println!("err: {:?}", from_utf8(&buf).unwrap_or_default());
    //         }
    //     }
    // }
}
