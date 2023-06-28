use smol_db_common::db_list::DBList;
use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_response::DBPacketResponse;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{fs, thread};

type DBListThreadSafe = Arc<RwLock<DBList>>;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8222").expect("Failed to bind to port 8222.");

    let mut thread_vec: Vec<JoinHandle<()>> = vec![];

    let db_list: DBListThreadSafe = Arc::new(RwLock::new(DBList::load_db_list()));

    let _ = fs::create_dir("./data");

    fs::read_dir("./data").expect("Data directory ./data must exist"); // the data directory must exist, so we make sure this happens

    // control-c handler for saving things before the server shuts down.
    let db_list_clone_ctrl_c = Arc::clone(&db_list);
    ctrlc::set_handler(move || {
        println!("Received CTRL+C, gracefully shutting down program.");
        let lock = db_list_clone_ctrl_c.read().unwrap();
        println!("{:?}", lock.list.read().unwrap());
        lock.save_db_list();
        lock.save_all_db();
        println!("Saved all db files and db list.");
        exit(0);
    })
    .unwrap();

    // thread that continuously checks if caches need to be removed from cache when they get old.
    let cache_invalidator_thread_db_list = Arc::clone(&db_list);
    let cache_invalidator_thread = thread::spawn(move || loop {
        let invalidated_caches = cache_invalidator_thread_db_list
            .read()
            .unwrap()
            .sleep_caches();

        cache_invalidator_thread_db_list
            .read()
            .unwrap()
            .save_all_db();
        cache_invalidator_thread_db_list
            .read()
            .unwrap()
            .save_db_list();

        if invalidated_caches > 0 {
            let number_of_caches_remaining = cache_invalidator_thread_db_list
                .read()
                .unwrap()
                .cache
                .read()
                .unwrap()
                .len() as u32;
            println!(
                "Slept {} caches, {} caches remain in cache.",
                invalidated_caches, number_of_caches_remaining
            );
        }

        thread::sleep(Duration::from_secs(10));
    });

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
            let stream = income.expect("Failed to receive tcp stream");
            handle_client(stream, db_list_clone);
        });

        thread_vec.push(handle);
        println!(
            "New client connected, current number of connected clients: {}",
            thread_vec.len()
        );
    }

    for handle in thread_vec {
        handle.join().unwrap();
    }
    cache_invalidator_thread.join().unwrap();
}

fn handle_client(mut stream: TcpStream, db_list: DBListThreadSafe) {
    let mut buf: [u8; 1024] = [0; 1024];
    let mut client_key = String::new();
    loop {
        // client loop

        let read_result = stream.read(&mut buf);

        if let Ok(read) = read_result {
            if read != 0 {
                #[cfg(debug_assertions)]
                println!("read size: {}", read); // this is a debug print
                let response = match DBPacket::deserialize_packet(&buf[0..read]) {
                    Ok(pack) => {
                        #[cfg(debug_assertions)]
                        println!("packet data: {:?}", pack); // this is also a debug print
                        match pack {
                            DBPacket::Read(db_name, db_location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.read_db(&db_name, &db_location, &client_key);
                                resp
                            }
                            DBPacket::Write(db_name, db_location, db_write_value) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.write_db(
                                    &db_name,
                                    &db_location,
                                    db_write_value,
                                    &client_key,
                                );
                                db_list.read().unwrap().save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::CreateDB(db_name, db_settings) => {
                                let lock = db_list.read().unwrap();
                                let resp =
                                    lock.create_db(db_name.get_db_name(), db_settings, &client_key);
                                lock.save_db_list();
                                lock.save_all_db();
                                resp
                            }
                            DBPacket::DeleteDB(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.delete_db(db_name.get_db_name(), &client_key);
                                db_list.read().unwrap().save_db_list();
                                resp
                            }
                            DBPacket::ListDB => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.list_db();
                                resp
                            }
                            DBPacket::ListDBContents(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.list_db_contents(&db_name, &client_key);
                                resp
                            }
                            DBPacket::AddAdmin(db_name, admin_hash) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.add_admin(&db_name, admin_hash, &client_key);
                                lock.save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::AddUser(db_name, user_hash) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.add_user(&db_name, user_hash, &client_key);
                                lock.save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::SetKey(key) => {
                                let lock = db_list.read().unwrap();
                                if lock.super_admin_hash_list.read().unwrap().is_empty() {
                                    // if there are no super admins, the first person to log in is the super admin.
                                    let mut super_admin_list_lock =
                                        lock.super_admin_hash_list.write().unwrap();
                                    super_admin_list_lock.push(key.clone());
                                }
                                client_key = key;
                                DBPacketResponse::SuccessNoData
                            }
                            DBPacket::GetDBSettings(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.get_db_settings(&db_name, &client_key);
                                resp
                            }
                            DBPacket::ChangeDBSettings(db_name, db_settings) => {
                                let lock = db_list.read().unwrap();
                                let resp =
                                    lock.change_db_settings(&db_name, db_settings, &client_key);
                                lock.save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::GetRole(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.get_role(&db_name, &client_key);
                                resp
                            }
                            DBPacket::DeleteData(db_name, db_location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.delete_data(&db_name, &db_location, &client_key);
                                lock.save_specific_db(&db_name);
                                resp
                            }
                        }
                    }
                    Err(err) => {
                        println!("packet serialization error: {}", err);
                        continue;
                    }
                };

                let ser = serde_json::to_string(&response).unwrap();
                let write_result = stream.write(ser.as_bytes());

                if write_result.is_err() {
                    println!("Client dropped. Unable to write socket data. {:?}", stream);
                    break;
                }
            } else {
                println!("Client dropped. Read 0 bytes from socket. {:?}", stream);
                break;
            }
        } else {
            println!("Client dropped. Unable to read socket data. {:?}", stream);
            break;
        }
    }
}
