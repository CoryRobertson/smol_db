#![allow(unused_variables,dead_code)] // TODO: remove this lints

use std::fs::{File};
use std::io::{Read, Write};
use std::net::{TcpListener};
use std::str::from_utf8;
use std::sync::{Arc, RwLock};
use std::{fs, thread};
use std::collections::HashMap;
use std::thread::JoinHandle;
use std::time::SystemTime;
use smol_db_common::{DBContent, DBPacket, DBPacketInfo};
use serde::Serialize;
use serde::Deserialize;


// TODO: move these structs and impl blocks to lib.rs

#[derive(Serialize,Deserialize,Debug,Clone)]
struct DBList {
    //TODO: store the cache and list in an RWLock, and eventually store each DB in the cache in an RWLock so individual databases can be read from and written to concurrently.
    //  These should allow us to read/write from each individual database concurrently.
    //  Something like RWLock<HashMap<DBPacketInfo,RWLock<DB>>>
    //  And RWLock<Vec<DBPacketInfo>>
    list: Vec<DBPacketInfo>, // vector of strings containing file names of the databases.
    cache: HashMap<DBPacketInfo,DB>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
struct DB {
    db_content: DBContent,
    last_access_time: SystemTime,
}


// struct DBRead(String);

impl DBList {
    fn create_db(&mut self, db_name: &str) -> std::io::Result<File> {
        let mut res = File::create(db_name);

        if let Ok(file) = &mut res {
            let db_packet_info = DBPacketInfo::new(db_name);
            let db = DB { db_content: DBContent::default(), last_access_time: SystemTime::now() };
            let ser = serde_json::to_string(&db.db_content).unwrap();
            let _ = file.write(ser.as_ref()).expect("TODO: panic message");
            self.cache.insert(db_packet_info.clone(), db);
            self.list.push(db_packet_info);

        }

        res
    }
    fn delete_db(&mut self, db_name: &str) -> std::io::Result<()> {
        let res = fs::remove_file(db_name);

        if res.is_ok() {
            let db_packet_info = DBPacketInfo::new(db_name);
            self.cache.remove(&db_packet_info);
            let index_res = self.list.binary_search(&db_packet_info);
            if let Ok(index) = index_res {
                self.list.remove(index);
            }
        }

        res
    }
    fn read_db(&mut self, read_pack: &DBPacket) -> Result<String,()> {
        return match read_pack {
            DBPacket::Read(p_info, p_location) => {
                if let Some(db) = self.cache.get_mut(p_info) {
                    // cache was hit
                    db.last_access_time = SystemTime::now();

                    Ok(db.db_content.read_from_db(p_location.as_key()).unwrap().to_string())
                } else if self.list.contains(p_info) {
                    // cache was missed but the db exists on the file system

                    let mut db_file = File::open(p_info.get_db_name()).unwrap();
                    let mut db_content_string = String::new();
                    db_file.read_to_string(&mut db_content_string).expect("TODO: panic message");
                    let db_content: DBContent = DBContent::read_ser_data(db_content_string).unwrap();

                    let return_value = db_content.read_from_db(p_location.as_key()).expect("RETURN VALUE DID NOT EXIST").clone();

                    let db = DB { db_content, last_access_time: SystemTime::now() };
                    self.cache.insert(p_info.clone(), db);


                    Ok(return_value)
                } else {
                    // cache was neither hit, nor did the db exist on the file system
                    Err(())
                }
            }
            DBPacket::Write(_, _, _) => { Err(()) }
            DBPacket::CreateDB(_) => { Err(()) }
            DBPacket::DeleteDB(_) => { Err(()) }
        };
    }

    fn write_db(&mut self, write_pack: &DBPacket) -> Result<String,()> {
        return match write_pack {
            DBPacket::Read(_, _) => { Err(()) }
            DBPacket::Write(db_info, db_location, db_data) => {
                if let Some(db) = self.cache.get_mut(db_info) {
                    // cache is hit, db is currently loaded
                    db.last_access_time = SystemTime::now();
                    return match db.db_content.content.insert(db_location.as_key().to_string(),db_data.get_data().to_string()) {
                        None => {
                            // if the db insertion had no previous value, simply return an empty string, this could be updated later possibly.
                            Ok("".to_string())
                        }
                        Some(updated_value) => {
                            // if the db insertion had a previous value, return it.
                            Ok(updated_value)
                        }
                    }
                } else if self.list.contains(db_info) {
                    // cache was missed, but the requested database did in fact exist
                    let mut db_file = File::open(db_info.get_db_name()).unwrap();
                    let mut db_content_string = String::new();
                    db_file.read_to_string(&mut db_content_string).expect("TODO: panic message");
                    let db_content: DBContent = DBContent::read_ser_data(db_content_string).unwrap();

                    let mut db = DB { db_content, last_access_time: SystemTime::now() };
                    let returned_value = match db.db_content.content.insert(db_location.as_key().to_string(),db_data.get_data().to_string()) {
                        None => { "".to_string() }
                        Some(updated_value) => {
                            updated_value
                        }
                    };
                    self.cache.insert(db_info.clone(), db);

                    Ok(returned_value)
                } else {
                    Err(())
                }
            }
            DBPacket::CreateDB(_) => { Err(()) }
            DBPacket::DeleteDB(_) => { Err(()) }
        }
    }

    //TODO:  finish implementing for write_db
}

fn main() {
    println!("Hello, world!");


    let listener = TcpListener::bind("0.0.0.0:8222").unwrap();

    let mut thread_vec: Vec<JoinHandle<()>> = vec![];

    let db_list = Arc::new(RwLock::new(DBList{ list: vec![], cache: Default::default() }));

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
            let mut buf: [u8 ; 1024] = [0 ; 1024];
            loop {
                // client loop

                let read_result = stream.read(&mut buf);

                if let Ok(read) = read_result {
                    // println!("{:?}",buf);
                    let s = from_utf8(&buf[0..read]).unwrap();
                    // println!("this is s: [{}]",s);
                    let pack: DBPacket = serde_json::from_str(s).unwrap();
                    println!("{:?}", pack);
                }
                let write_result = stream.write("test".as_bytes());

                if read_result.is_err() || write_result.is_err() {
                    break;
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

// fn handle_client(stream: &TcpStream) -> bool {
//     true
// }