#![allow(unused_variables,dead_code)] // TODO: remove this lints

use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use smol_db_common::{DBPacket};

fn main() {

    let mut buf: [u8 ; 1024] = [0 ; 1024];

    let packet1 = DBPacket::new_create_db("test1");
    let packet2 = DBPacket::new_delete_db("test1");
    let packet3 = DBPacket::new_read("test1", "location1");
    let packet4 = DBPacket::new_write("test1", "location1");

    let pack = packet1.serialize_packet().unwrap();


    println!("dassad: {:?}",pack);

    let mut client = TcpStream::connect("localhost:8222").unwrap();



    let _ = client.write(pack.as_bytes());
    let _ = client.read(&mut buf);
    println!("{:?}", from_utf8(&buf).unwrap_or_default());


    // let _ = client.read(&mut buf);
    // println!("{:?}", from_utf8(&buf).unwrap_or_default());
    // let _ = client.write("4321".as_bytes());
}
