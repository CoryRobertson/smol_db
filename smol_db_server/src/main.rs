use smol_db_common::db_list::DBList;
use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_response::DBPacketResponse;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

type DBListThreadSafe = Arc<RwLock<DBList>>;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8222").unwrap();

    let mut thread_vec: Vec<JoinHandle<()>> = vec![];

    let db_list: DBListThreadSafe = Arc::new(RwLock::new(DBList::load_db_list()));

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
            .invalidate_caches();

        if invalidated_caches > 0 {
            let number_of_caches_remaining = cache_invalidator_thread_db_list
                .read()
                .unwrap()
                .cache
                .read()
                .unwrap()
                .len() as u32;
            println!(
                "Invalidated caches: {}, {} caches remain in cache.",
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
                println!("read size: {}", read); // this is a debug print
                let response = match DBPacket::deserialize_packet(&buf[0..read]) {
                    Ok(pack) => {
                        println!("packet data: {:?}", pack); // this is also a debug print
                        match pack {
                            DBPacket::Read(db_name, db_location) => {
                                let lock = db_list.read().unwrap();

                                let resp = lock.read_db(&db_name, &db_location, &client_key);
                                println!("{:?}", resp);
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
                                println!("{:?}", resp);
                                db_list.read().unwrap().save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::CreateDB(db_name, db_settings) => {
                                let lock = db_list.read().unwrap();

                                let resp =
                                    lock.create_db(db_name.get_db_name(), db_settings, &client_key);
                                println!("{:?}", resp);
                                db_list.read().unwrap().save_db_list();
                                resp
                            }
                            DBPacket::DeleteDB(db_name) => {
                                let lock = db_list.read().unwrap();

                                // only allow db deletion when key is super admin
                                let resp = lock.delete_db(db_name.get_db_name(), &client_key);
                                println!("{:?}", resp);
                                db_list.read().unwrap().save_db_list();
                                resp
                            }
                            DBPacket::ListDB => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.list_db();
                                println!("{:?}", resp);
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
                                resp
                            }
                            DBPacket::AddUser(db_name, user_hash) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.add_user(&db_name, user_hash, &client_key);
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
                // let rand_num: u32 = rand::thread_rng().gen_range(0..100);
                // let reply_test = format!("test{}", rand_num);
                // let reply_bytes = reply_test.as_bytes();
                let write_result = stream.write(ser.as_bytes());

                if write_result.is_err() {
                    println!("Client dropped. Unable to write socket data.");
                    break;
                }
            } else {
                println!("Client dropped. Read 0 bytes from socket.");
                break;
            }
        } else {
            println!("Client dropped. Unable to read socket data.");
            break;
        }
    }
}
