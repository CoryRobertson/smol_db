//! Binary application that runs a `smol_db` server instance
use smol_db_common::db_list::DBList;
use smol_db_common::db_packets::db_packet::DBPacket;
use smol_db_common::db_packets::db_packet_response::DBPacketResponseError::BadPacket;
use smol_db_common::db_packets::db_packet_response::DBSuccessResponse;
#[cfg(feature = "logging")]
use smol_db_common::{
    logging::log_entry::LogEntry, logging::log_level::LogLevel, logging::log_message::LogMessage,
    logging::logger::Logger,
};
#[cfg(not(feature = "no-saving"))]
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
#[cfg(feature = "logging")]
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
#[cfg(not(feature = "no-saving"))]
use std::time::Duration;
use rsa::rand_core::OsRng;
use rsa::RsaPublicKey;
use smol_db_common::encryption::encrypted_data::EncryptedData;
use smol_db_common::prelude::{DBPacketResponseError, SuccessNoData};

type DBListThreadSafe = Arc<RwLock<DBList>>;

#[cfg(feature = "logging")]
const LOG_FILE_PATH: &str = "./data/log.log";

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8222").expect("Failed to bind to port 8222.");

    {
        print!("Features enabled:");

        #[cfg(feature = "statistics")]
        print!(" Statistics");
        #[cfg(feature = "logging")]
        print!(" Logging");
        #[cfg(feature = "no-saving")]
        print!(" No-Saving");
        println!();
    }

    #[cfg(feature = "logging")]
    let logger = Arc::new(Logger::new(PathBuf::from(LOG_FILE_PATH)).unwrap());

    let mut thread_vec: Vec<JoinHandle<()>> = vec![];

    let db_list: DBListThreadSafe = Arc::new(RwLock::new(DBList::load_db_list()));

    #[cfg(not(feature = "no-saving"))]
    let _ = fs::create_dir("./data");

    #[cfg(not(feature = "no-saving"))]
    fs::read_dir("./data").expect("Data directory ./data must exist"); // the data directory must exist, so we make sure this happens

    // control-c handler for saving things before the server shuts down.
    let db_list_clone_ctrl_c = Arc::clone(&db_list);
    #[cfg(feature = "logging")]
    let ctrl_c_logger_clone = Arc::clone(&logger);
    ctrlc::set_handler(move || {
        println!("Received CTRL+C, gracefully shutting down program.");
        #[cfg(feature = "logging")]
        let _ = ctrl_c_logger_clone.log(&LogEntry::new(
            LogMessage::new("Received CTRL+C, gracefully shutting down program."),
            LogLevel::Info,
        ));
        let lock = db_list_clone_ctrl_c.read().unwrap();
        println!("{:?}", lock.list.read().unwrap());

        #[cfg(not(feature = "no-saving"))]
        {
            lock.save_db_list();
            lock.save_all_db();
            println!("Saved all db files and db list.");
        }
        #[cfg(feature = "logging")]
        ctrl_c_logger_clone
            .log(&LogEntry::new(
                LogMessage::new("Saved all db files and db list."),
                LogLevel::Info,
            ))
            .expect("Failed to log saving message to log file");
        exit(0);
    })
    .unwrap();

    // thread that continuously checks if caches need to be removed from cache when they get old.
    #[cfg(not(feature = "no-saving"))]
    let cache_invalidator_thread_db_list = Arc::clone(&db_list);
    #[cfg(feature = "logging")]
    #[allow(unused_variables)]
    let cache_invalidator_logger = Arc::clone(&logger);
    let cache_invalidator_thread = thread::spawn(move || {
        #[cfg(not(feature = "no-saving"))]
        loop {
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
                    .len();
                let msg = format!(
                    "Slept {} caches, {} caches remain in cache.",
                    invalidated_caches, number_of_caches_remaining
                );
                println!("{}", msg);
                #[cfg(feature = "logging")]
                let _ = cache_invalidator_logger.log(&LogEntry::new(
                    LogMessage::new(msg.as_str()),
                    LogLevel::Info,
                ));
            }

            thread::sleep(Duration::from_secs(10));
        }
    });

    println!("Waiting for connections on port 8222");

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
        #[cfg(feature = "logging")]
        let logger_clone = Arc::clone(&logger);
        let handle = thread::spawn(move || {
            let stream = income.expect("Failed to receive tcp stream");

            #[cfg(feature = "logging")]
            let msg = {
                match stream
                    .peer_addr()
                    .map(|socket| format!("{}", socket))
                    .map_err(|err| format!("{:?}", err))
                {
                    Ok(s) => s,
                    Err(s) => s,
                }
            };

            #[cfg(feature = "logging")]
            let _ = logger_clone.log(&LogEntry::new(
                LogMessage::new(format!("New client connected: {}", msg).as_str()),
                LogLevel::Info,
            ));
            #[cfg(feature = "logging")]
            handle_client(stream, &db_list_clone, logger_clone);
            #[cfg(not(feature = "logging"))]
            handle_client(stream, &db_list_clone);
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

#[allow(clippy::let_and_return)]
fn handle_client(
    mut stream: TcpStream,
    db_list: &DBListThreadSafe,
    #[cfg(feature = "logging")] logger: Arc<Logger>,
) {
    let ip_address = stream.peer_addr().unwrap();
    let mut buf: [u8; 1024] = [0; 1024];
    let mut client_key = String::new();

    let mut client_name = format!("Client [{}] [{}]:", ip_address, client_key);

    let mut rng = OsRng::default();

    let mut client_pub_key_opt: Option<RsaPublicKey> = None;

    loop {
        // client loop

        let read_result = stream.read(&mut buf);

        if let Ok(read) = read_result {
            if read != 0 {
                #[cfg(debug_assertions)]
                println!("read size: {}", read); // this is a debug print
                let response = match DBPacket::deserialize_packet(&buf[0..read]) {
                    Ok(mut pack) => {
                        #[cfg(debug_assertions)]
                        println!("packet data: {:?}", pack); // this is also a debug print

                        match &pack {
                            DBPacket::Encrypted(data) => {
                                let unencrypted_data = db_list.read().unwrap().server_key.decrypt_packet(&data).unwrap();
                                pack = unencrypted_data;
                            }
                            _ => {}
                        }

                        match pack {
                            DBPacket::PubKey(key) => {
                                client_pub_key_opt = Some(key);
                                Ok(SuccessNoData)
                            }
                            DBPacket::Encrypted(_) => {
                                Err(BadPacket)
                            }
                            DBPacket::Read(db_name, db_location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.read_db(&db_name, &db_location, &client_key);
                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} read \"{}\" in \"{}\", response: {:?}",
                                            client_name, db_location, db_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));
                                resp
                            }
                            DBPacket::Write(db_name, db_location, db_write_value) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.write_db(
                                    &db_name,
                                    &db_location,
                                    &db_write_value.clone(),
                                    &client_key,
                                );

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} wrote \"{}\" to \"{}\" in \"{}\", response: {:?}",
                                            client_name, db_write_value, db_location, db_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                #[cfg(not(feature = "no-saving"))]
                                db_list.read().unwrap().save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::CreateDB(db_name, db_settings) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.create_db(
                                    db_name.get_db_name(),
                                    db_settings.clone(),
                                    &client_key,
                                );
                                #[cfg(not(feature = "no-saving"))]
                                lock.save_db_list();

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(format!("{} created database \"{}\" with settings \"{:?}\", response: {:?}",client_name,db_name,db_settings, resp).as_str()),
                                    LogLevel::Info
                                ));

                                #[cfg(not(feature = "no-saving"))]
                                lock.save_all_db();
                                resp
                            }
                            DBPacket::DeleteDB(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.delete_db(db_name.get_db_name(), &client_key);

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} deleted database \"{}\", response: {:?}",
                                            client_name, db_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                #[cfg(not(feature = "no-saving"))]
                                db_list.read().unwrap().save_db_list();
                                resp
                            }
                            DBPacket::ListDB => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.list_db();

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} listed databases, response: {:?}",
                                            client_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                resp
                            }
                            DBPacket::ListDBContents(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.list_db_contents(&db_name, &client_key);

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} listed database contents of \"{}\", response: {:?}",
                                            client_name, db_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                resp
                            }
                            DBPacket::AddAdmin(db_name, admin_hash) => {
                                let lock = db_list.read().unwrap();
                                let resp =
                                    lock.add_admin(&db_name, admin_hash.clone(), &client_key);

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} added an admin \"{}\" to \"{}\", response: {:?}",
                                            client_name, admin_hash, db_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                #[cfg(not(feature = "no-saving"))]
                                lock.save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::AddUser(db_name, user_hash) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.add_user(&db_name, user_hash.clone(), &client_key);

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} added an admin \"{}\" to \"{}\" response: {:?}",
                                            client_name, user_hash, db_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                #[cfg(not(feature = "no-saving"))]
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

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!("{} set key to \"{}\"", client_name, key).as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                client_key = key;
                                client_name = format!("Client [{}] [{}]:", ip_address, client_key);
                                Ok(DBSuccessResponse::SuccessNoData)
                            }
                            DBPacket::GetDBSettings(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.get_db_settings(&db_name, &client_key);

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} got db settings from \"{}\", response: {:?}",
                                            client_name, db_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                resp
                            }
                            DBPacket::ChangeDBSettings(db_name, db_settings) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.change_db_settings(
                                    &db_name,
                                    db_settings.clone(),
                                    &client_key,
                                );

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(format!("{} changed db settings of \"{}\" to \"{:?}\", response: {:?}",client_name,db_name,db_settings,resp).as_str()),
                                    LogLevel::Info
                                ));

                                #[cfg(not(feature = "no-saving"))]
                                lock.save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::GetRole(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.get_role(&db_name, &client_key);

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} got role from \"{}\", response: {:?}",
                                            client_name, db_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                resp
                            }
                            DBPacket::DeleteData(db_name, db_location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.delete_data(&db_name, &db_location, &client_key);

                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} deleted data from \"{}\" in \"{}\", response: {:?}",
                                            client_name, db_name, db_location, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));

                                #[cfg(not(feature = "no-saving"))]
                                lock.save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::GetStats(db_name) => {
                                db_list.read().unwrap().get_stats(&db_name, &client_key)
                            }
                        }
                    }
                    Err(err) => {
                        println!("packet serialization error: {}", err);
                        Err(BadPacket)
                        // continue;
                    }
                };

                let write_result;
                if let Some(pubkey_client) = &client_pub_key_opt {
                    let ser_resp = serde_json::to_string(&response).unwrap();

                    let packet_ser = DBPacket::Encrypted(EncryptedData::new(ser_resp.as_bytes()));

                    let ency = smol_db_common::encryption::encrypt(pubkey_client,&mut rng,packet_ser.serialize_packet().unwrap().as_bytes()).unwrap();

                    write_result = stream.write(ency.as_slice());
                } else {
                    let ser = serde_json::to_string(&response).unwrap();
                    write_result = stream.write(ser.as_bytes());
                };

                if write_result.is_err() {
                    println!(
                        "{} dropped. Unable to write socket data. {:?}",
                        client_name, stream
                    );
                    break;
                }
            } else {
                println!(
                    "{} dropped. Read 0 bytes from socket. {:?}",
                    client_name, stream
                );
                break;
            }
        } else {
            println!(
                "{} dropped. Unable to read socket data. {:?}",
                client_name, stream
            );
            break;
        }
    }
}
