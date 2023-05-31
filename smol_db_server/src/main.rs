#![allow(unused_variables, dead_code)] // TODO: remove this lints

use smol_db_common::{DBList, DBPacket};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::str::from_utf8;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

fn main() {
    println!("Hello, world!");

    let listener = TcpListener::bind("0.0.0.0:8222").unwrap();

    let mut thread_vec: Vec<JoinHandle<()>> = vec![];

    let db_list = Arc::new(RwLock::new(DBList {
        list: vec![],
        cache: Default::default(),
    }));

    for income in listener.incoming() {
        for i in 0..thread_vec.len() {
            match thread_vec.get(i) {
                None => {}
                Some(t) => {
                    if t.is_finished() {
                        thread_vec.remove(i);
                    }
                }
            }
        }

        let handle = thread::spawn(move || {
            let mut stream = income.expect("failed to receive tcp stream");
            let mut buf: [u8; 1024] = [0; 1024];
            loop {
                // client loop

                let read_result = stream.read(&mut buf);

                if let Ok(read) = read_result {
                    if read != 0 {
                        println!("read size: {}", read); // this is a debug print
                        let s = from_utf8(&buf[0..read]).unwrap();
                        let pack: DBPacket = serde_json::from_str(s).unwrap();
                        println!("packet data: {:?}", pack); // this is also a debug print
                        match pack {
                            // TODO: implement these blocks
                            DBPacket::Read(_, _) => {}
                            DBPacket::Write(_, _, _) => {}
                            DBPacket::CreateDB(_) => {}
                            DBPacket::DeleteDB(_) => {}
                        }
                    }
                }
                let write_result = stream.write("test".as_bytes());

                if read_result.is_err() || write_result.is_err() {
                    println!("client dropped.");
                    break;
                }
            }
        });

        thread_vec.push(handle);
        println!(
            "connection handled. number of connections: {}",
            thread_vec.len()
        );
    }

    for handle in thread_vec {
        handle.join().unwrap();
    }
}
