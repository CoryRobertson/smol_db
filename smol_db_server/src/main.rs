use smol_db_common::db_list::DBList;
use smol_db_common::db_packets::db_packet::DBPacket;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

type DBListThreadSafe = Arc<RwLock<DBList>>;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8222").unwrap();

    let mut thread_vec: Vec<JoinHandle<()>> = vec![];

    let db_list: DBListThreadSafe = Arc::new(RwLock::new(DBList::load_db_list()));

    // TODO: remove databases from cache when time exceeds a number in the struct.

    // TODO: save databases on program shutdown.

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

        let db_list_clone = Arc::clone(&db_list);
        let handle = thread::spawn(move || {
            let stream = income.expect("failed to receive tcp stream");
            handle_client(stream, db_list_clone);
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

fn handle_client(mut stream: TcpStream, db_list: DBListThreadSafe) {
    let mut buf: [u8; 1024] = [0; 1024];
    let mut received: Vec<u8> = vec![];
    loop {
        // client loop

        let read_result = loop {
            let bytes_read = stream.read(&mut buf);

            match bytes_read {
                Ok(read_count) => {
                    received.extend_from_slice(&buf[..read_count]);
                    println!("ada");
                    if read_count <= 1024 {
                        break Ok(received.len());
                    }
                }
                Err(err) => {
                    break Err(err);
                }
            }
        };

        if let Ok(read) = read_result {
            if read != 0 {
                println!("read size: {}", read); // this is a debug print
                let response = match DBPacket::deserialize_packet(&received[0..read]) {
                    Ok(pack) => {
                        println!("packet data: {:?}", pack); // this is also a debug print
                        match pack {
                            DBPacket::Read(db_name, db_location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.read_db(db_name, db_location);
                                println!("{:?}", resp);
                                received.clear();
                                resp
                            }
                            DBPacket::Write(db_name, db_location, db_write_value) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.write_db(&db_name, &db_location, db_write_value);
                                println!("{:?}", resp);
                                db_list.read().unwrap().save_specific_db(&db_name);
                                received.clear();
                                resp
                            }
                            DBPacket::CreateDB(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.create_db(db_name.get_db_name());
                                println!("{:?}", resp);
                                db_list.read().unwrap().save_db_list();
                                received.clear();
                                resp
                            }
                            DBPacket::DeleteDB(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.delete_db(db_name.get_db_name());
                                println!("{:?}", resp);
                                db_list.read().unwrap().save_db_list();
                                received.clear();
                                resp
                            }
                        }

                    }
                    Err(err) => {
                        println!("{:?}", received);
                        println!("packet serialization error: {}", err);
                        continue;
                    }
                };

                let ser = serde_json::to_string(&response).unwrap();
                let write_result = stream.write(ser.as_bytes());

                if write_result.is_err() {
                    println!("Client dropped. Unable to write socket data.");
                    break;
                }
            } else {
                println!("Client dropped. Unable to read 0 bytes");
                break;
            }
        } else {
            println!("Client dropped. Unable to read socket data.");
            break;
        }
    }
}
