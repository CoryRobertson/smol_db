use crate::DBListThreadSafe;
use smol_db_common::prelude::DBPacketResponseError::BadPacket;
use smol_db_common::prelude::{DBPacket, RsaPublicKey, SuccessNoData, SuccessReply};
#[cfg(feature = "logging")]
use smol_db_common::{
    logging::log_entry::LogEntry, logging::log_level::LogLevel, logging::log_message::LogMessage,
    logging::logger::Logger,
};
use std::io::{Read, Write};
use std::net::TcpStream;
#[cfg(feature = "logging")]
use std::sync::Arc;

#[allow(clippy::let_and_return)]
pub(crate) async fn handle_client(
    mut stream: TcpStream,
    db_list: DBListThreadSafe,
    #[cfg(feature = "logging")] logger: Arc<Logger>,
) {
    let ip_address = stream.peer_addr().unwrap();
    let mut buf: [u8; 1024] = [0; 1024];
    let mut client_key = String::new();

    let mut client_name = format!("Client [{}] [{}]:", ip_address, client_key);

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

                        // overwrite the packet with the unencrypted version if it is encrypted
                        if let DBPacket::Encrypted(data) = &pack {
                            #[cfg(debug_assertions)]
                            println!("Received encrypted data: {:?}", data);
                            let unencrypted_data = db_list
                                .read()
                                .unwrap()
                                .server_key
                                .decrypt_client_packet(data)
                                .unwrap();
                            pack = unencrypted_data;
                            #[cfg(debug_assertions)]
                            println!("Unencrypted data: {:?}", pack);
                        }

                        match pack {
                            DBPacket::SetupEncryption => {
                                // non standard conforming implementation of sending a response back, the client is expected to understand this given they requested to establish encryption
                                let key = db_list.read().unwrap().server_key.get_pub_key().clone();
                                let ser = serde_json::to_string(&key).unwrap();
                                let resp = Ok(SuccessReply(ser));
                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} requested to setup encryption, response: {:?}",
                                            client_name, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));
                                resp
                            }
                            DBPacket::PubKey(key) => {
                                let resp = Ok(SuccessNoData);
                                #[cfg(feature = "logging")]
                                let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} sent pub-key {:?} response: {:?}",
                                            client_name, key, resp
                                        )
                                        .as_str(),
                                    ),
                                    LogLevel::Info,
                                ));
                                client_pub_key_opt = Some(key);
                                resp
                            }
                            DBPacket::Encrypted(_) => {
                                #[cfg(feature = "logging")]
                                    let _ = logger.log(&LogEntry::new(
                                    LogMessage::new(
                                        format!(
                                            "{} sent encrypted packet that was not handled properly, report this on github in the issues section of smol_db",
                                            client_name,
                                        )
                                            .as_str(),
                                    ),
                                    LogLevel::Error,
                                ));
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
                                Ok(SuccessNoData)
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

                let ser = serde_json::to_string(&response).unwrap();

                // check if the client is using encryption in their communication
                let write_result = match &client_pub_key_opt {
                    None => {
                        // client is not using encryption, send the raw bytes
                        stream.write(ser.as_bytes())
                    }
                    Some(key) => {
                        // client is using encryption, encrypt the packet then send the encrypted bytes
                        let ency_data = db_list
                            .write()
                            .unwrap()
                            .server_key
                            .encrypt_packet(&ser, key)
                            .unwrap();
                        stream.write(ency_data.get_data())
                    }
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
