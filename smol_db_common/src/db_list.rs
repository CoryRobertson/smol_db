#![allow(clippy::expect_fun_call)]
//! Contains structs and implementations for managing the active list of databases, that are both in filesystem, and in cache.
//! Also handles what to do when packets are received that modify any database that does or does not exist.
use crate::db::Role::SuperAdmin;
use crate::db::DB;
use crate::db_data::DBData;
use crate::db_packets::db_location::DBLocation;
use crate::db_packets::db_packet_info::DBPacketInfo;
#[cfg(not(feature = "statistics"))]
use crate::db_packets::db_packet_response::DBPacketResponseError::BadPacket;
use crate::db_packets::db_packet_response::DBPacketResponseError::{
    DBFileSystemError, DBNotFound, InvalidPermissions, SerializationError, UserNotFound,
    ValueNotFound,
};
use crate::db_packets::db_packet_response::DBSuccessResponse::{SuccessNoData, SuccessReply};
use crate::db_packets::db_packet_response::{DBPacketResponseError, DBSuccessResponse};
use crate::db_packets::db_settings::DBSettings;
use crate::encryption::server_encrypt::ServerKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::RwLock;
use std::time::SystemTime;
use tracing::{debug, error, info, warn};
use crate::db_content::DBContent;
use crate::prelude::DBPacket;

#[derive(Serialize, Deserialize, Debug)]
/// `DBList` represents a server that takes requests and handles them on a given `smol_db` server.
/// This struct can be used to create a local only database as well, by simply instantiating it and not listening for socket requests.
pub struct DBList {
    /// Vector of DBPacketInfo's containing file names of the databases that are available to be read from.
    pub list: RwLock<Vec<DBPacketInfo>>,

    /// Hashmap that takes a DBPacketInfo and returns the database corresponding to the name in the given packet.
    #[serde(skip)]
    pub cache: RwLock<HashMap<DBPacketInfo, RwLock<DB>>>,

    /// Vector containing the list of super admins on the server. Super admins have non-restricted access to all parts of the server.
    pub super_admin_hash_list: RwLock<Vec<String>>,

    #[serde(skip)]
    /// Server key used for encryption when the user requests end to end encryption
    pub server_key: ServerKey,
}

impl DBList {

    fn handle_stream(&self, client_stream: &mut TcpStream, db_table: &DBContent) -> Result<(), DBPacketResponseError> {
        for item in db_table.content.iter() {
            let mut buf: [u8; 1024] = [0; 1024];
            debug!("Waiting for client to await next item");
            let mut read_client = String::new();
            client_stream.read(&mut buf).unwrap();

            read_client = String::from_utf8(buf.to_vec()).unwrap();

            match serde_json::from_str::<DBPacket>(&read_client) {
                Ok(packet) => {
                    debug!("Packet read: {:?}", packet);
                    if !matches!(packet,DBPacket::ReadyForNextItem) {
                        return Err(DBPacketResponseError::BadPacket);
                    }
                }
                Err(err) => {
                    error!("err: {}",err);
                }
            }

            debug!("Client requested next item");

            let _ = client_stream.write(item.0.as_bytes()).map_err(|err| {
                error!("{}", err);
                DBPacketResponseError::StreamClosedUnexpectedly
            })?;
            let _ = client_stream.write(item.1.as_bytes()).map_err(|err| {
                error!("{}", err);
                DBPacketResponseError::StreamClosedUnexpectedly
            })?;
            info!("Wrote key value pair to stream");
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn stream_table(
        &self,
        packet: &DBPacketInfo,
        client_key: &String,
        client_stream: &mut TcpStream,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        let super_admin_list = self.get_super_admin_list();
        let list_lock = self.list.read().unwrap();

        if let Some(db) = self.cache.read().unwrap().get(&packet) {
            info!("DB Cache hit");
            // cache was hit
            db.write().unwrap().update_access_time();

            let db_lock = db.read().unwrap();

            if db_lock.has_read_permissions(&client_key, &super_admin_list) {
                let db_table = db_lock.get_content().clone();
                drop(db_lock);

                let s: Result<DBSuccessResponse<String>, DBPacketResponseError> = Ok(SuccessNoData);
                let starting_packet = serde_json::to_string(&s).unwrap();
                client_stream.write(starting_packet.as_bytes()).unwrap();

                let _ = self.handle_stream(client_stream,&db_table)?;

                return Ok(SuccessNoData);
            } else {
                return Err(InvalidPermissions);
            };
        }

        if list_lock.contains(&packet) {
            info!("DB Cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(&packet)?;

            db.update_access_time();

            if db.has_read_permissions(&client_key, &super_admin_list) {

                let db_table = db.get_content();
                let s: Result<DBSuccessResponse<String>, DBPacketResponseError> = Ok(SuccessNoData);
                let starting_packet = serde_json::to_string(&s).unwrap();
                client_stream.write(starting_packet.as_bytes()).unwrap();

                for item in db_table.content.iter() {
                    let mut buf: [u8; 1024] = [0; 1024];
                    debug!("Waiting for client to await next item");
                    let mut read_client = String::new();
                    client_stream.read(&mut buf).unwrap();

                    read_client = String::from_utf8(buf.to_vec()).unwrap();

                    match serde_json::from_str::<DBPacket>(&read_client) {
                        Ok(packet) => {
                            debug!("Packet read: {:?}", packet);
                            if !matches!(packet,DBPacket::ReadyForNextItem) {
                                return Err(DBPacketResponseError::BadPacket);
                            }
                        }
                        Err(err) => {
                            error!("err: {}",err);
                        }
                    }

                    // if !serde_json::from_str::<DBPacket>(&read_client).is_ok_and(|packet| {
                    //     debug!("Packet read: {:?}", packet);
                    //     matches!(packet,DBPacket::ReadyForNextItem) }) {
                    //     return Err(DBPacketResponseError::BadPacket);
                    // }

                    debug!("Client requested next item");

                    let _ = client_stream.write(item.0.as_bytes()).map_err(|err| {
                        error!("{}", err);
                        DBPacketResponseError::StreamClosedUnexpectedly
                    })?;
                    let _ = client_stream.write(item.1.as_bytes()).map_err(|err| {
                        error!("{}", err);
                        DBPacketResponseError::StreamClosedUnexpectedly
                    })?;
                    info!("Wrote key value pair to stream");
                }
            } else {
                return Err(InvalidPermissions);
            };

            self.cache
                .write()
                .unwrap()
                .insert(packet.clone(), RwLock::from(db));

            return Ok(SuccessNoData);
        } else {
            // cache was neither hit, nor did the db exist on the file system
            return Err(DBNotFound);
        }
    }

    // TODO: open stream function handler should be about here, it should return an iterator over an entire table,
    //  probably cloning the table at the start so the lock on the table can be dropped quickly?
    // TODO: we probably want a "streaming read" and a "streaming write" function and packet system

    /// Returns true if the given hash is a super admin hash
    #[tracing::instrument(skip(self))]
    pub fn is_super_admin(&self, hash: &String) -> bool {
        self.super_admin_hash_list.read().unwrap().contains(hash)
    }

    /// Returns the super admin list
    #[tracing::instrument(skip(self))]
    fn get_super_admin_list(&self) -> Vec<String> {
        self.super_admin_hash_list.read().unwrap().clone()
    }

    #[allow(unused_variables)]
    #[allow(clippy::ptr_arg)]
    /// Returns the db stats used for a given database when permissions allow the user to read them
    #[tracing::instrument(skip(self))]
    pub fn get_stats(
        &self,
        p_info: &DBPacketInfo,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        #[cfg(not(feature = "statistics"))]
        {
            warn!("Statistics packet received, however statistics is not enabled on this server");
            return Err(BadPacket);
        }

        #[cfg(feature = "statistics")]
        {
            let super_admin_list = self.get_super_admin_list();

            let list_lock = self.list.read().unwrap();
            if let Some(db) = self.cache.read().unwrap().get(p_info) {
                info!("DB Cache hit");
                // cache was hit
                let mut db_lock = db.write().unwrap();

                db_lock.update_access_time();

                return if db_lock.get_role(client_key, &super_admin_list).is_admin() {
                    serde_json::to_string(db_lock.get_statistics())
                        .map(SuccessReply)
                        .map_err(|_| SerializationError)
                } else {
                    Err(InvalidPermissions)
                };
            }

            return if list_lock.contains(p_info) {
                info!("DB Cache missed");
                // cache was missed but the db exists on the file system

                let mut db = Self::read_db_from_file(p_info)?;

                db.update_access_time();

                let resp = if db.get_role(client_key, &super_admin_list).is_admin() {
                    serde_json::to_string(db.get_statistics())
                        .map(SuccessReply)
                        .map_err(|_| SerializationError)
                } else {
                    Err(InvalidPermissions)
                };

                self.cache
                    .write()
                    .unwrap()
                    .insert(p_info.clone(), RwLock::from(db));

                resp
            } else {
                // cache was neither hit, nor did the db exist on the file system
                info!("Database not found {}", p_info);
                Err(DBNotFound)
            };
        }
    }

    /// Deletes the given data from a db if the user has write permissions
    #[tracing::instrument(skip(self))]
    pub fn delete_data(
        &self,
        p_info: &DBPacketInfo,
        db_location: &DBLocation,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        let super_admin_list = self.get_super_admin_list();

        let list_lock = self.list.read().unwrap();
        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            info!("DB Cache hit");
            // cache was hit
            let mut db_lock = db.write().unwrap();

            db_lock.update_access_time();

            return if db_lock.has_write_permissions(client_key, &super_admin_list) {
                db_lock
                    .get_content_mut()
                    .content
                    .remove(db_location.as_key())
                    .map(SuccessReply)
                    .ok_or(ValueNotFound)
            } else {
                Err(InvalidPermissions)
            };
        }

        return if list_lock.contains(p_info) {
            info!("DB Cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(p_info)?;

            db.update_access_time();

            let resp = if db.has_write_permissions(client_key, &super_admin_list) {
                db.get_content_mut()
                    .content
                    .remove(db_location.as_key())
                    .map(SuccessReply)
                    .ok_or(ValueNotFound)
            } else {
                Err(InvalidPermissions)
            };

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            resp
        } else {
            // cache was neither hit, nor did the db exist on the file system
            info!("Database not found {}", p_info);
            Err(DBNotFound)
        };
    }

    /// Responds with the role of the client key inside a given db, if they are a super admin, the result is always a super admin role.
    #[tracing::instrument(skip(self))]
    pub fn get_role(
        &self,
        p_info: &DBPacketInfo,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        let super_admin_list = self.get_super_admin_list();

        if super_admin_list.contains(client_key) {
            info!("User was super admin");
            // early return super admin if their key is a super admin key.
            return Ok(SuccessReply(serde_json::to_string(&SuperAdmin).unwrap()));
        }

        let list_lock = self.list.read().unwrap();

        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            info!("DB Cache hit");
            // cache was hit
            let mut db_lock = db.write().unwrap();

            db_lock.update_access_time();

            let serialized_role =
                serde_json::to_string(&db_lock.get_role(client_key, &super_admin_list)).unwrap();

            return Ok(SuccessReply(serialized_role));
        }

        return if list_lock.contains(p_info) {
            info!("DB Cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(p_info)?;

            db.update_access_time();

            let serialized_role =
                serde_json::to_string(&db.get_role(client_key, &super_admin_list)).unwrap();

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            Ok(SuccessReply(serialized_role))
        } else {
            // cache was neither hit, nor did the db exist on the file system
            info!("Database not found {}", p_info);
            Err(DBNotFound)
        };
    }

    /// Replaces `DBSettings` for a given DB, requires super admin privileges.
    /// Returns `SuccessNoData` when successful
    #[tracing::instrument(skip(self))]
    pub fn change_db_settings(
        &self,
        p_info: &DBPacketInfo,
        new_db_settings: DBSettings,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        if !self.is_super_admin(client_key) {
            // change settings requires super admin, early return if the user is not a super admin
            info!("User was not super admin");
            return Err(InvalidPermissions);
        }

        let list_lock = self.list.read().unwrap();
        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            info!("DB cache hit");
            // cache was hit
            let mut db_lock = db.write().unwrap();

            db_lock.update_access_time();

            db_lock.set_settings(new_db_settings);
            drop(db_lock);
            return Ok(SuccessNoData);
        }

        return if list_lock.contains(p_info) {
            info!("DB cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(p_info)?;

            db.update_access_time();

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            Ok(SuccessNoData)
        } else {
            // cache was neither hit, nor did the db exist on the file system
            info!("Database not found {}", p_info);
            Err(DBNotFound)
        };
    }

    /// Returns the `DBSettings` serialized as a string
    /// Only super admins can get the db settings
    #[tracing::instrument(skip(self))]
    pub fn get_db_settings(
        &self,
        p_info: &DBPacketInfo,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        if !self.is_super_admin(client_key) {
            info!("Client is not super admin");
            // change settings requires super admin, early return if the user is not a super admin
            return Err(InvalidPermissions);
        }

        let list_lock = self.list.read().unwrap();
        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            info!("DB Cache hit");

            // cache was hit
            let mut db_lock = db.write().unwrap();

            db_lock.update_access_time();

            return serde_json::to_string(&db_lock.get_settings())
                .map(SuccessReply)
                .map_err(|_| SerializationError);
        }

        return if list_lock.contains(p_info) {
            info!("DB Cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(p_info)?;

            db.update_access_time();

            let response = serde_json::to_string(&db.get_settings())
                .map(SuccessReply)
                .map_err(|_| SerializationError);

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            response
        } else {
            // cache was neither hit, nor did the db exist on the file system
            Err(DBNotFound)
        };
    }

    /// Adds a user to a given DB, requires admin privileges or super admin privileges.
    #[tracing::instrument(skip(self))]
    pub fn add_user(
        &self,
        p_info: &DBPacketInfo,
        new_key: String,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        let list_lock = self.list.read().unwrap();
        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            info!("DB Cache hit");
            // cache was hit
            let mut db_lock = db.write().unwrap();

            return if db_lock.get_settings().is_admin(client_key) || self.is_super_admin(client_key)
            {
                db_lock.update_access_time();

                db_lock.get_settings_mut().add_user(new_key);
                Ok(SuccessNoData)
            } else {
                Err(InvalidPermissions)
            };
        }

        return if list_lock.contains(p_info) {
            info!("DB Cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(p_info)?;

            db.update_access_time();

            let response =
                if db.get_settings().is_admin(client_key) || self.is_super_admin(client_key) {
                    db.get_settings_mut().add_admin(new_key);
                    Ok(SuccessNoData)
                } else {
                    Err(InvalidPermissions)
                };

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            response
        } else {
            // cache was neither hit, nor did the db exist on the file system
            Err(DBNotFound)
        };
    }

    /// Removes a user from a given DB, requires admin privileges
    #[tracing::instrument(skip(self))]
    pub fn remove_user(
        &self,
        p_info: &DBPacketInfo,
        removed_key: &str,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        let list_lock = self.list.read().unwrap();
        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            info!("DB Cache hit");
            // cache was hit
            let mut db_lock = db.write().unwrap();

            return if db_lock.get_settings().is_admin(client_key) || self.is_super_admin(client_key)
            {
                db_lock.update_access_time();

                if db_lock.get_settings_mut().remove_user(removed_key) {
                    Ok(SuccessNoData)
                } else {
                    Err(UserNotFound)
                }
            } else {
                Err(InvalidPermissions)
            };
        }

        return if list_lock.contains(p_info) {
            info!("DB Cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(p_info)?;

            db.update_access_time();

            let response =
                if db.get_settings().is_admin(client_key) || self.is_super_admin(client_key) {
                    if db.get_settings_mut().remove_user(removed_key) {
                        Ok(SuccessNoData)
                    } else {
                        Err(UserNotFound)
                    }
                } else {
                    Err(InvalidPermissions)
                };

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            response
        } else {
            // cache was neither hit, nor did the db exist on the file system
            Err(DBNotFound)
        };
    }

    /// Remove an admin from given DB, requires super admin permissions.
    #[tracing::instrument(skip(self))]
    pub fn remove_admin(
        &self,
        p_info: &DBPacketInfo,
        removed_key: &str,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        if !self.is_super_admin(client_key) {
            // change settings requires super admin, early return if the user is not a super admin
            return Err(InvalidPermissions);
        }

        let list_lock = self.list.read().unwrap();
        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            info!("DB Cache hit");
            // cache was hit
            let mut db_lock = db.write().unwrap();

            db_lock.update_access_time();

            return if db_lock.get_settings_mut().remove_admin(removed_key) {
                Ok(SuccessNoData)
            } else {
                Err(UserNotFound)
            };
        }

        return if list_lock.contains(p_info) {
            info!("DB Cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(p_info)?;

            db.update_access_time();

            let response = {
                if db.get_settings_mut().remove_admin(removed_key) {
                    Ok(SuccessNoData)
                } else {
                    Err(UserNotFound)
                }
            };

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            response
        } else {
            // cache was neither hit, nor did the db exist on the file system
            Err(DBNotFound)
        };
    }

    /// Adds an admin to a given database, requires super admin permissions to perform.
    #[tracing::instrument(skip(self))]
    pub fn add_admin(
        &self,
        p_info: &DBPacketInfo,
        hash: String,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        if !self.is_super_admin(client_key) {
            info!("User is not a super admin");
            // to add an admin, you must be a super admin first, else you have invalid permissions
            return Err(InvalidPermissions);
        }

        let list_lock = self.list.read().unwrap();
        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            info!("DB Cache hit");
            // cache was hit
            let mut db_lock = db.write().unwrap();
            db_lock.update_access_time();

            db_lock.get_settings_mut().add_admin(hash);
            drop(db_lock);
            return Ok(SuccessNoData);
        }

        return if list_lock.contains(p_info) {
            info!("DB Cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(p_info)?;

            db.update_access_time();
            db.get_settings_mut().add_admin(hash);

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            Ok(SuccessNoData)
        } else {
            // cache was neither hit, nor did the db exist on the file system
            Err(DBNotFound)
        };
    }

    /// Removes all caches which last access time exceeds their invalidation time.
    /// Read locks the cache list, will Write lock the cache list if there are caches to be removed.
    /// Returns the number of caches removed.
    #[tracing::instrument(skip_all)]
    pub fn sleep_caches(&self) -> usize {
        // prepare a list of invalid caches
        let invalid_cache_names: Vec<DBPacketInfo> = {
            let read_lock = self.cache.read().unwrap();
            read_lock
                .iter()
                // filter to keep only caches that have a last access duration greater than their invalidation time.
                .filter(|(_, db)| {
                    let db_lock = db.read().unwrap();
                    let last_access_time = db_lock.get_access_time();
                    let invalidation_time = db_lock.get_settings().get_invalidation_time();
                    drop(db_lock);

                    match SystemTime::now().duration_since(last_access_time) {
                        // invalidate them based on their duration since access and invalidation time
                        Ok(duration_since_access) => duration_since_access >= invalidation_time,
                        // if there is some sort of duration error, simply don't invalidate them
                        Err(_) => false,
                    }
                })
                .map(|(db_name, _)| db_name.clone()) // there has to be a way to get rid of this clone -_-
                .collect()
        };
        info!("DB sleep list: {:?}", invalid_cache_names);
        info!("Putting {} databases to sleep", invalid_cache_names.len());

        if !invalid_cache_names.is_empty() {
            // only write lock the cache if we have caches to remove.
            let mut write_lock = self.cache.write().unwrap();
            for invalid_cache_name in &invalid_cache_names {
                info!("DB being put to sleep: {}", invalid_cache_name);
                write_lock.remove(invalid_cache_name);
            }
        }
        invalid_cache_names.len()
    }

    /// Saves all db instances to a file.
    #[tracing::instrument(skip_all)]
    pub fn save_all_db(&self) {
        info!("Saving all databases");
        let list = self.cache.read().unwrap();
        for (db_name, db) in list.iter() {
            let mut db_file = match File::create(format!("./data/{}", db_name.get_db_name())) {
                Ok(f) => {
                    info!("DB file created for DB: {}", db_name);
                    f
                }
                Err(e) => {
                    let log_message =
                        format!("Unable to create db file: {}, {}", db_name.get_db_name(), e);
                    error!("{}", log_message);
                    panic!("{}", log_message);
                }
            };

            let db_lock = db.read().unwrap();
            let ser = match serde_json::to_string(&db_lock.clone()) {
                Ok(s) => {
                    info!("Successfully serialized database");
                    s
                }
                Err(e) => {
                    let log_message = format!(
                        "Unable to serialize db file: {}, {}",
                        db_name.get_db_name(),
                        e
                    );
                    error!("{}", log_message);
                    panic!("{}", log_message)
                }
            };
            match db_file.write(ser.as_bytes()) {
                Ok(len) => {
                    info!("Successfully wrote {} to file with size: {}", db_name, len);
                }
                Err(e) => {
                    let log_message = format!(
                        "Unable to write to db file: {}, {}",
                        db_name.get_db_name(),
                        e
                    );
                    error!("{}", log_message);
                    panic!("{}", log_message);
                }
            }
        }
    }

    /// Saves a specific db by name to file.
    /// Read locks the cache.
    #[tracing::instrument(skip(self))]
    pub fn save_specific_db(&self, db_name: &DBPacketInfo) {
        let list = self.cache.read().unwrap();
        match list.get(db_name) {
            Some(db_lock) => {
                info!("Database exists, saving to file");
                let mut db_file = File::create(format!("./data/{}", db_name.get_db_name())).expect(
                    &format!("Unable to create db file: {}", db_name.get_db_name()),
                );
                let db_clone = db_lock.read().unwrap().clone();
                let ser = serde_json::to_string(&db_clone).unwrap();
                let _ = db_file.write(ser.as_bytes()).expect(&format!(
                    "Unable to write to db file: {}",
                    db_name.get_db_name()
                ));
                info!("Database successfully saved");
            }
            None => {
                let log_message = format!(
                    "Unable to save db: {}, db not found in list?",
                    db_name.get_db_name()
                );
                error!("{}", log_message);
                panic!("{}", log_message);
            }
        }
    }

    /// Saves all db names to a file.
    #[tracing::instrument(skip_all)]
    pub fn save_db_list(&self) {
        info!("Saving database list");
        let mut db_list_file =
            File::create("./data/db_list.ser").expect("Unable to save db_list.ser");
        let ser_data = serde_json::to_string(&self).expect("Unable to serialize self.");
        let _ = db_list_file
            .write(ser_data.as_bytes())
            .expect("Unable to write bytes to db_list.ser");
        info!("Successfully saved database list");
    }

    /// Loads all db names from the db list file.
    #[tracing::instrument]
    pub fn load_db_list() -> Self {
        info!("Loading database list");
        match File::open("./data/db_list.ser") {
            Ok(mut f) => {
                // file found, load from file data
                let mut ser = String::new();
                f.read_to_string(&mut ser)
                    .expect("Unable to read db_list.ser to string");
                let db_list: Self =
                    serde_json::from_str(&ser).expect("Unable to deserialize db_list.ser");
                info!("Successfully opened database list and deserialized");
                db_list
            }
            Err(e) => {
                warn!("No database list found, making one. This could be an error or is the first startup of the server. {}",e);
                // no file found, load default
                Self::default()
            }
        }
    }

    /// Returns true if the given db exists.
    #[tracing::instrument(skip(self))]
    fn db_name_exists(&self, db_name: &str) -> bool {
        self.list
            .read()
            .unwrap()
            .contains(&DBPacketInfo::new(db_name))
    }

    /// Creates a DB given a name, the packet is not needed, only the name.
    /// Requires super admin privileges
    #[tracing::instrument(skip(self))]
    pub fn create_db(
        &self,
        db_name: &str,
        db_settings: DBSettings,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        if !self.is_super_admin(client_key) {
            // to create a db you must be a super admin
            return Err(InvalidPermissions);
        }

        if self.db_name_exists(db_name) {
            return Err(DBPacketResponseError::DBAlreadyExists);
        }

        let mut list_write_lock = self.list.write().unwrap();

        return match File::open(format!("./data/{}", db_name)) {
            Ok(_) => {
                // db file was found and should not have been, because this db already exists

                Err(DBPacketResponseError::DBAlreadyExists)
            }
            Err(_) => {
                // db file was not found
                match File::create(format!("./data/{}", db_name)) {
                    Ok(mut file) => {
                        let mut cache_write_lock = self.cache.write().unwrap();
                        let db_packet_info = DBPacketInfo::new(db_name);
                        let db = DB::new_from_settings(db_settings);
                        let ser = serde_json::to_string(&db).unwrap();
                        let _ = file
                            .write(ser.as_ref())
                            .expect(&format!("Unable to write db to file. {}", db_name));
                        cache_write_lock.insert(db_packet_info.clone(), RwLock::from(db));
                        list_write_lock.push(db_packet_info);
                        drop(cache_write_lock);
                        info!("Successfully created DB file");
                        Ok(SuccessNoData)
                    }
                    Err(e) => {
                        // db file was unable to be created
                        error!("Unable to create DB file: {}", e);
                        Err(DBFileSystemError)
                    }
                }
            }
        };
    }

    /// Handles deleting a db, given a name for the db. Removes the database given a name, and deletes the corresponding file.
    /// If the file is successfully removed, the db is also removed from the cache, and list.
    #[tracing::instrument(skip(self))]
    pub fn delete_db(
        &self,
        db_name: &str,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        if !self.is_super_admin(client_key) {
            // to delete a db, you must be a super admin no matter what.
            return Err(InvalidPermissions);
        }

        if !self.db_name_exists(db_name) {
            return Err(DBNotFound);
        }

        let mut list_lock = self.list.write().unwrap();

        let mut cache_lock = self.cache.write().unwrap();

        match fs::remove_file(format!("./data/{}", db_name)) {
            Ok(_) => {
                let db_packet_info = DBPacketInfo::new(db_name);
                cache_lock.remove(&db_packet_info);

                let mut removed = false;
                let it = list_lock.clone();
                for (index, item) in it.into_iter().enumerate() {
                    if db_packet_info.get_db_name() == item.get_db_name() {
                        list_lock.remove(index);
                        removed = true;
                    }
                }

                if !removed {
                    // if no db was removed from the list, then we should tell the user that this deletion failed in some way.
                    return Err(DBFileSystemError);
                }

                info!("Successfully deleted database: {}", db_name);
                Ok(SuccessNoData)
            }
            Err(e) => {
                error!("Unable to delete database file: {}", e);
                Err(DBFileSystemError)
            }
        }
    }

    /// Reads a db from a db packet info.
    /// Err on db not existing as a file: `DBFileSystemError`
    #[tracing::instrument]
    fn read_db_from_file(p_info: &DBPacketInfo) -> Result<DB, DBPacketResponseError> {
        let mut db_file = match File::open(format!("./data/{}", p_info.get_db_name())) {
            Ok(f) => f,
            Err(e) => {
                error!("Unable to read database from file: {}", e);
                // early return db file system error when no file was able to be opened, should never happen due to the db file being in a list of known working db files.
                return Err(DBFileSystemError);
            }
        };

        let mut db_content_string = String::new();
        db_file
            .read_to_string(&mut db_content_string)
            .expect("TODO: panic message");
        let db: DB = serde_json::from_str(&db_content_string).unwrap_or_default();
        Ok(db)
    }

    /// Reads a database given a packet, returns the value if it was found.
    #[tracing::instrument(skip(self))]
    pub fn read_db(
        &self,
        p_info: &DBPacketInfo,
        p_location: &DBLocation,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        let super_admin_list = self.get_super_admin_list();

        let list_lock = self.list.read().unwrap();

        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            info!("DB Cache hit");
            // cache was hit
            db.write().unwrap().update_access_time();

            let db_lock = db.read().unwrap();

            return if db_lock.has_read_permissions(client_key, &super_admin_list) {
                db_lock
                    .get_content()
                    .read_from_db(p_location.as_key())
                    .map(|value| SuccessReply(value.to_string()))
                    .ok_or(ValueNotFound)
            } else {
                Err(InvalidPermissions)
            };
        }

        if list_lock.contains(p_info) {
            info!("DB Cache missed");
            // cache was missed but the db exists on the file system

            let mut db = Self::read_db_from_file(p_info)?;

            db.update_access_time();

            let response = if db.has_read_permissions(client_key, &super_admin_list) {
                let return_value = db
                    .get_content()
                    .read_from_db(p_location.as_key())
                    .expect("RETURN VALUE DID NOT EXIST")
                    .clone();
                Ok(SuccessReply(return_value))
            } else {
                Err(InvalidPermissions)
            };

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            response
        } else {
            // cache was neither hit, nor did the db exist on the file system
            Err(DBNotFound)
        }
    }

    /// Writes to a db given a `DBPacket`
    #[tracing::instrument(skip(self))]
    pub fn write_db(
        &self,
        db_info: &DBPacketInfo,
        db_location: &DBLocation,
        db_data: &DBData,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        let super_admin_list = self.get_super_admin_list();

        let list_lock = self.list.read().unwrap();

        {
            // scope the cache lock so it goes out of scope faster, allowing us to get a write lock later.
            let cache_lock = self.cache.read().unwrap();

            if let Some(db) = cache_lock.get(db_info) {
                info!("DB Cache hit");
                // cache is hit, db is currently loaded

                let mut db_lock = db.write().unwrap();

                return if db_lock.has_write_permissions(client_key, &super_admin_list) {
                    db_lock.update_access_time();
                    Ok(db_lock
                        .get_content_mut()
                        .content
                        .insert(
                            db_location.as_key().to_string(),
                            db_data.get_data().to_string(),
                        )
                        .map_or(SuccessNoData, SuccessReply))
                } else {
                    Err(InvalidPermissions)
                };
            }
        }

        if list_lock.contains(db_info) {
            info!("DB Cache missed");
            // cache was missed, but the requested database did in fact exist

            let mut cache_lock = self.cache.write().unwrap();

            let mut db = Self::read_db_from_file(db_info)?;

            db.update_access_time();

            if db.has_write_permissions(client_key, &super_admin_list) {
                let returned_value = db
                    .get_content_mut()
                    .content
                    .insert(
                        db_location.as_key().to_string(),
                        db_data.get_data().to_string(),
                    )
                    .map_or(SuccessNoData, SuccessReply);

                cache_lock.insert(db_info.clone(), RwLock::from(db));

                Ok(returned_value)
            } else {
                cache_lock.insert(db_info.clone(), RwLock::from(db));
                Err(InvalidPermissions)
            }
        } else {
            Err(DBNotFound)
        }
    }

    /// Returns the db list in a serialized form of Vec : `DBPacketInfo`
    #[tracing::instrument(skip(self))]
    pub fn list_db(&self) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        let list = self.list.read().unwrap();
        serde_json::to_string(&list.clone())
            .map(SuccessReply)
            .map_err(|_| SerializationError)
    }

    /// Returns the db contents in a serialized form of HashMap<String, String>
    #[tracing::instrument(skip(self))]
    pub fn list_db_contents(
        &self,
        db_info: &DBPacketInfo,
        client_key: &String,
    ) -> Result<DBSuccessResponse<String>, DBPacketResponseError> {
        if !self.db_name_exists(db_info.get_db_name()) {
            return Err(DBNotFound);
        }

        let super_admin_list = self.get_super_admin_list();

        let list_lock = self.list.read().unwrap();

        {
            // scope the cache lock so it goes out of scope faster, allowing us to get a write lock later.
            let cache_lock = self.cache.read().unwrap();

            if let Some(db) = cache_lock.get(db_info) {
                info!("DB Cache hit");
                // cache is hit, db is currently loaded

                let mut db_lock = db.write().unwrap();

                return if db_lock.has_list_permissions(client_key, &super_admin_list)
                    || self.is_super_admin(client_key)
                {
                    db_lock.update_access_time();

                    serde_json::to_string(&db_lock.get_content().content)
                        .map(SuccessReply)
                        .map_err(|_| SerializationError)
                } else {
                    Err(InvalidPermissions)
                };
            }
        }

        if list_lock.contains(db_info) {
            info!("DB Cache missed");
            // cache was missed, but the requested database did in fact exist

            let mut cache_lock = self.cache.write().unwrap();

            let mut db = Self::read_db_from_file(db_info)?;

            if db.has_list_permissions(client_key, &super_admin_list) {
                db.update_access_time();

                let returned_value = &db.get_content().content;

                let output_response = serde_json::to_string(returned_value)
                    .map(SuccessReply)
                    .map_err(|_| SerializationError);
                cache_lock.insert(db_info.clone(), RwLock::from(db));

                output_response
            } else {
                db.update_access_time();

                cache_lock.insert(db_info.clone(), RwLock::from(db));

                Err(InvalidPermissions)
            }
        } else {
            Err(DBNotFound)
        }
    }
}

impl Default for DBList {
    #[tracing::instrument]
    fn default() -> Self {
        Self {
            list: RwLock::new(vec![]),
            cache: RwLock::new(HashMap::new()),
            super_admin_hash_list: RwLock::new(vec![]),
            server_key: ServerKey::new().unwrap(),
        }
    }
}
