use crate::DBListThreadSafe;
use smol_db_common::prelude::DBPacketResponseError::BadPacket;
use smol_db_common::prelude::{DBPacket, RsaPublicKey, SuccessNoData, SuccessReply};
use std::io::{Read, Write};
use std::net::TcpStream;
use tracing::{debug, error, info, warn};

#[allow(clippy::let_and_return)]
#[tracing::instrument(skip(db_list))]
pub async fn handle_client(mut stream: TcpStream, db_list: DBListThreadSafe) {
    info!("New client connected");
    let ip_address = stream.peer_addr().unwrap();
    let mut buf: [u8; 1024] = [0; 1024];
    let mut client_key = String::new();

    let mut client_name = format!("Client [{}] [{}]:", ip_address, client_key);

    let mut client_pub_key_opt: Option<RsaPublicKey> = None;

    loop {
        // client loop

        info!("Awaiting packet information from: {}", client_name);
        let read_result = stream.read(&mut buf);

        if let Ok(read) = read_result {
            if read != 0 {
                debug!("Read size: {}", read);
                let response = match DBPacket::deserialize_packet(&buf[0..read]) {
                    Ok(mut pack) => {
                        debug!("Packet data: {:?}", pack);

                        // overwrite the packet with the unencrypted version if it is encrypted
                        if let DBPacket::Encrypted(data) = &pack {
                            debug!("Received encrypted data: {:?}", data);
                            let unencrypted_data = db_list
                                .read()
                                .unwrap()
                                .server_key
                                .decrypt_client_packet(data)
                                .unwrap();
                            pack = unencrypted_data;

                            debug!("Unencrypted data: {:?}", pack);
                        }

                        match pack {
                            DBPacket::GetListLength(p_info, location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.get_db_list_length(&p_info, &location, &client_key);
                                info!(
                                    "{} got list length in {} using {:?}, response: {:?} ",
                                    client_name, p_info, location, resp
                                );
                                resp
                            }
                            DBPacket::ClearList(p_info, location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.clear_db_list(&p_info, &location, &client_key);
                                info!(
                                    "{} cleared list in {} using {:?}, response: {:?} ",
                                    client_name, p_info, location, resp
                                );
                                resp
                            }
                            DBPacket::AddToList(p_info, location, data) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.add_to_db_list_content(
                                    &p_info,
                                    &location,
                                    &data,
                                    &client_key,
                                );
                                info!(
                                    "{} wrote to list in {} using {:?}, response: {:?} ",
                                    client_name, p_info, location, resp
                                );
                                resp
                            }
                            DBPacket::RemoveFromList(p_info, location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.remove_from_db_list_content(
                                    &p_info,
                                    &location,
                                    &client_key,
                                );
                                info!(
                                    "{} removed item from list in {} using {:?}, response: {:?} ",
                                    client_name, p_info, location, resp
                                );
                                resp
                            }
                            DBPacket::ReadFromList(p_info, location) => {
                                let lock = db_list.read().unwrap();
                                let resp =
                                    lock.read_from_db_list_content(&p_info, &location, &client_key);
                                info!(
                                    "{} read list from {} using {:?}, response: {:?} ",
                                    client_name, p_info, location, resp
                                );
                                resp
                            }
                            DBPacket::StreamList(p_info, location) => {
                                let lock = db_list.read().unwrap();
                                info!("Client beginning stream");
                                let resp = lock.stream_table_list(
                                    &p_info,
                                    &location,
                                    &client_key,
                                    &mut stream,
                                );
                                info!(
                                    "{} streamed \"{}\" list: {:?}, response: {:?}",
                                    client_name, p_info, location, resp
                                );

                                resp
                            }
                            DBPacket::EndStreamRead => {
                                warn!("Client requested to end stream when no stream was active: {}, {:?}", client_name, pack);
                                // its possible we receive this packet after a stream is read all the way to its end,
                                // meaning the user didn't know the stream ended, this is perfectly ok, we just don't respond.
                                continue;
                            }
                            DBPacket::ReadyForNextItem => {
                                warn!("Client requested stream item when no stream was active: {}, {:?}", client_name, pack);
                                // user requested next item when there was no item left in stream, this is ok it seems ?

                                Err(BadPacket)
                            }
                            DBPacket::StreamReadDb(packet) => {
                                let lock = db_list.read().unwrap();
                                info!("Client beginning stream");
                                let resp = lock.stream_table(&packet, &client_key, &mut stream);
                                info!(
                                    "{} streamed \"{}\", response: {:?}",
                                    client_name, packet, resp
                                );

                                resp
                            }
                            DBPacket::SetupEncryption => {
                                // non standard conforming implementation of sending a response back, the client is expected to understand this given they requested to establish encryption
                                let key = db_list.read().unwrap().server_key.get_pub_key().clone();
                                let ser = serde_json::to_string(&key).unwrap();
                                let resp = Ok(SuccessReply(ser));
                                info!(
                                    "{} requested to setup encryption, response: {:?}",
                                    client_name, resp
                                );
                                resp
                            }
                            DBPacket::PubKey(key) => {
                                let resp = Ok(SuccessNoData);
                                info!(
                                    "{} sent pub-key {:?} response: {:?}",
                                    client_name, key, resp
                                );
                                client_pub_key_opt = Some(key);
                                resp
                            }
                            DBPacket::Encrypted(_) => {
                                warn!("{} sent encrypted packet that was not handled properly, report this on github in the issues section of smol_db",client_name);
                                Err(BadPacket)
                            }
                            DBPacket::Read(db_name, db_location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.read_db(&db_name, &db_location, &client_key);
                                info!(
                                    "{} read \"{}\" in \"{}\", response: {:?}",
                                    client_name, db_location, db_name, resp
                                );
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

                                info!(
                                    "{} wrote \"{}\" to \"{}\" in \"{}\", response: {:?}",
                                    client_name, db_write_value, db_location, db_name, resp
                                );

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

                                info!("{} created database \"{}\" with settings \"{:?}\", response: {:?}",client_name,db_name,db_settings, resp);

                                #[cfg(not(feature = "no-saving"))]
                                lock.save_all_db();
                                resp
                            }
                            DBPacket::DeleteDB(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.delete_db(db_name.get_db_name(), &client_key);

                                info!(
                                    "{} deleted database \"{}\", response: {:?}",
                                    client_name, db_name, resp
                                );

                                #[cfg(not(feature = "no-saving"))]
                                db_list.read().unwrap().save_db_list();
                                resp
                            }
                            DBPacket::ListDB => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.list_db();

                                info!("{} listed databases, response: {:?}", client_name, resp);

                                resp
                            }
                            DBPacket::ListDBContents(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.list_db_contents(&db_name, &client_key);

                                info!(
                                    "{} listed database contents of \"{}\", response: {:?}",
                                    client_name, db_name, resp
                                );

                                resp
                            }
                            DBPacket::AddAdmin(db_name, admin_hash) => {
                                let lock = db_list.read().unwrap();
                                let resp =
                                    lock.add_admin(&db_name, admin_hash.clone(), &client_key);

                                info!(
                                    "{} added an admin \"{}\" to \"{}\", response: {:?}",
                                    client_name, admin_hash, db_name, resp
                                );

                                #[cfg(not(feature = "no-saving"))]
                                lock.save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::AddUser(db_name, user_hash) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.add_user(&db_name, user_hash.clone(), &client_key);

                                info!(
                                    "{} added an admin \"{}\" to \"{}\" response: {:?}",
                                    client_name, user_hash, db_name, resp
                                );

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

                                info!("{} set key to \"{}\"", client_name, key);

                                client_key = key;
                                client_name = format!("Client [{}] [{}]:", ip_address, client_key);
                                Ok(SuccessNoData)
                            }
                            DBPacket::GetDBSettings(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.get_db_settings(&db_name, &client_key);

                                info!(
                                    "{} got db settings from \"{}\", response: {:?}",
                                    client_name, db_name, resp
                                );

                                resp
                            }
                            DBPacket::ChangeDBSettings(db_name, db_settings) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.change_db_settings(
                                    &db_name,
                                    db_settings.clone(),
                                    &client_key,
                                );

                                info!(
                                    "{} changed db settings of \"{}\" to \"{:?}\", response: {:?}",
                                    client_name, db_name, db_settings, resp
                                );

                                #[cfg(not(feature = "no-saving"))]
                                lock.save_specific_db(&db_name);
                                resp
                            }
                            DBPacket::GetRole(db_name) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.get_role(&db_name, &client_key);

                                info!(
                                    "{} got role from \"{}\", response: {:?}",
                                    client_name, db_name, resp
                                );

                                resp
                            }
                            DBPacket::DeleteData(db_name, db_location) => {
                                let lock = db_list.read().unwrap();
                                let resp = lock.delete_data(&db_name, &db_location, &client_key);

                                info!(
                                    "{} deleted data from \"{}\" in \"{}\", response: {:?}",
                                    client_name, db_name, db_location, resp
                                );

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
                        let packet_text = String::from_utf8_lossy(&buf[0..read]);
                        error!(
                            "packet serialization error: {}, packet string: {}",
                            err, packet_text
                        );
                        Err(BadPacket)
                        // continue;
                    }
                };

                let ser = serde_json::to_string(&response).unwrap();

                // check if the client is using encryption in their communication
                let write_result =
                    write_to_client(&mut stream, client_pub_key_opt.as_ref(), ser, &db_list);

                if write_result.is_err() {
                    info!(
                        "{} dropped. Unable to write socket data. {:?}",
                        client_name, stream
                    );
                    break;
                }
            } else {
                info!(
                    "{} dropped. Read 0 bytes from socket. {:?}",
                    client_name, stream
                );
                break;
            }
        } else {
            info!(
                "{} dropped. Unable to read socket data. {:?}",
                client_name, stream
            );
            break;
        }
    }
}

fn write_to_client(
    stream: &mut TcpStream,
    client_pub_key_opt: Option<&RsaPublicKey>,
    ser: String,
    db_list: &DBListThreadSafe,
) -> std::io::Result<usize> {
    match &client_pub_key_opt {
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
    }
}
