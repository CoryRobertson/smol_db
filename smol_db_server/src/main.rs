use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;
use std::thread::JoinHandle;
use bson::{from_bson, to_bson};
use smol_db_common::{DBPacket, DBPacketInfo};

fn main() {
    println!("Hello, world!");


    let listener = TcpListener::bind("0.0.0.0:8222").unwrap();

    let mut thread_vec: Vec<JoinHandle<()>> = vec![];

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
            let mut stream = income.expect("failed to recieve tcp stream");
            let mut buf: [u8 ; 1024] = [0 ; 1024];
            loop {
                // client loop
                let write_result = stream.write("test".as_bytes());
                let read_result = stream.read(&mut buf);
                if read_result.is_err() || write_result.is_err() {
                    break;
                }

                if let Ok(read) = read_result {
                    // println!("{:?}",buf);
                    let s = from_utf8(&buf[0..read]).unwrap();
                    println!("this is s: [{}]",s);
                    let pack: DBPacket = serde_json::from_str(s).unwrap();
                    println!("{:?}", pack);
                }

            }
        });

        thread_vec.push(handle);
        println!("connection handled. number of connections: {}", thread_vec.len());

    }

    for handle in thread_vec {
        handle.join().unwrap();
    }

}

fn handle_client(stream: &TcpStream) -> bool {
    true
}