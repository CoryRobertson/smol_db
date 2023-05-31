#![allow(unused_variables, dead_code)] // TODO: remove this lints

use smol_db_common::DBPacket;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;

fn main() {
    let mut buf: [u8; 1024] = [0; 1024];

    let packet1 = DBPacket::new_create_db("test1");
    let packet2 = DBPacket::new_delete_db("test1");
    let packet3 = DBPacket::new_read("test1", "location1");
    let packet4 = DBPacket::new_write("test1", "location1", "data1");

    let pack = packet1.serialize_packet().unwrap();

    println!("dassad: {:?}", pack);

    let mut client = TcpStream::connect("localhost:8222").unwrap();

    let b = pack.as_ref();
    let bs = from_utf8(b).unwrap();

    println!("b {:?}", b);
    println!("bs {:?}", bs);

    let _ = client.write(b);
    let read_res = client.read(&mut buf);
    match read_res {
        Ok(len) => {
            println!("ok: {:?}", from_utf8(&buf[0..len]).unwrap_or_default());
        }
        Err(_) => {
            println!("err: {:?}", from_utf8(&buf).unwrap_or_default());
        }
    }
}
