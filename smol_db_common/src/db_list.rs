//! Contains structs and implementations for managing the active list of databases, that are both in filesystem, and in cache.
//! Also handles what to do when packets are received that modify any database that does or does not exist.
use crate::db::DB;
use crate::db_content::DBContent;
use crate::db_data::DBData;
use crate::db_packets::db_location::DBLocation;
use crate::db_packets::db_packet_info::DBPacketInfo;
use crate::db_packets::db_packet_response::DBPacketResponse::{Error, SuccessNoData, SuccessReply};
use crate::db_packets::db_packet_response::DBPacketResponseError::{DBNotFound, InvalidPermissions, SerializationError};
use crate::db_packets::db_packet_response::{DBPacketResponse, DBPacketResponseError};
use crate::db_packets::db_settings::DBSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::RwLock;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct DBList {
    /// Vector of DBPacketInfo's containing file names of the databases that are available to be read from.
    pub list: RwLock<Vec<DBPacketInfo>>,

    /// Hashmap that takes a DBPacketInfo and returns the database corresponding to the name in the given packet.
    #[serde(skip)]
    pub cache: RwLock<HashMap<DBPacketInfo, RwLock<DB>>>,
    // TODO: a rwlock vec here contains the list of super admin access key hashes.

    /// Vector containing the list of super admins on the server. Super admins have non-restricted access to all parts of the server.
    pub super_admin_hash_list: RwLock<Vec<String>>,
}

impl DBList {
    // TODO: packet handler functions createdb, deletedb, readdb, writedb, should all begin taking in an access key hash in their function inputs.

    pub fn is_super_admin(&self, hash: &String) -> bool {
        self.super_admin_hash_list.read().unwrap().contains(hash)
    }

    pub fn add_user(&self, p_info: &DBPacketInfo, new_key: String, client_key: &String) -> DBPacketResponse<String> {
        let list_lock = self.list.read().unwrap();
        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            // cache was hit
            let mut db_lock = db.write().unwrap();

            return if db_lock.db_settings.is_admin(client_key) {
                db_lock.last_access_time = SystemTime::now();

                db_lock.db_settings.add_user(new_key);
                SuccessNoData
            } else {
                Error(InvalidPermissions)
            };
        }

        return if list_lock.contains(p_info) {
            // cache was missed but the db exists on the file system

            let mut db_file = match File::open(p_info.get_db_name()) {
                Ok(f) => f,
                Err(_) => {
                    // early return db file system error when no file was able to be opened, should never happen due to the db file being in a list of known working db files.
                    return Error(DBPacketResponseError::DBFileSystemError);
                }
            };

            let mut db_content_string = String::new();
            db_file
                .read_to_string(&mut db_content_string)
                .expect("TODO: panic message");
            let mut db: DB = serde_json::from_str(&db_content_string).unwrap();

            db.last_access_time = SystemTime::now();

            let response = if db.db_settings.is_admin(client_key) {
                db.db_settings.add_admin(new_key);
                SuccessNoData
            } else {
                Error(InvalidPermissions)
            };

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            response
        } else {
            // cache was neither hit, nor did the db exist on the file system
            Error(DBNotFound)
        }
    }

    pub fn add_admin(&self, p_info: &DBPacketInfo, hash: String, client_key: &String) -> DBPacketResponse<String> {

        if !self.is_super_admin(client_key) {
            // to add an admin, you must be a super admin first, else you have invalid permissions
            return Error(InvalidPermissions);
        }

        let list_lock = self.list.read().unwrap();
        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            // cache was hit
            let mut db_lock = db.write().unwrap();
            db_lock.last_access_time = SystemTime::now();


            db_lock.db_settings.add_admin(hash);
            return SuccessNoData;
        }

        return if list_lock.contains(p_info) {
            // cache was missed but the db exists on the file system

            let mut db_file = match File::open(p_info.get_db_name()) {
                Ok(f) => f,
                Err(_) => {
                    // early return db file system error when no file was able to be opened, should never happen due to the db file being in a list of known working db files.
                    return Error(DBPacketResponseError::DBFileSystemError);
                }
            };

            let mut db_content_string = String::new();
            db_file
                .read_to_string(&mut db_content_string)
                .expect("TODO: panic message");
            let mut db: DB = serde_json::from_str(&db_content_string).unwrap();

            db.last_access_time = SystemTime::now();
            db.db_settings.add_admin(hash);

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            SuccessNoData
        } else {
            // cache was neither hit, nor did the db exist on the file system
            Error(DBNotFound)
        }
    }

    /// Removes all caches which last access time exceeds their invalidation time.
    /// Read locks the cache list, will Write lock the cache list if there are caches to be removed.
    /// Returns the number of caches removed.
    pub fn invalidate_caches(&self) -> usize {
        // prepare a list of invalid caches
        let invalid_cache_names: Vec<DBPacketInfo> = {
            let read_lock = self.cache.read().unwrap();
            read_lock
                .iter()
                // filter to keep only caches that have a last access duration greater than their invalidation time.
                .filter(|(_, db)| {
                    let db_lock = db.read().unwrap();
                    let last_access_time = db_lock.last_access_time;
                    let invalidation_time = db_lock.db_settings.get_invalidation_time();

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

        if !invalid_cache_names.is_empty() {
            // only write lock the cache if we have caches to remove.
            let mut write_lock = self.cache.write().unwrap();
            for invalid_cache_name in &invalid_cache_names {
                write_lock.remove(invalid_cache_name);
            }
        }
        invalid_cache_names.len()
    }

    /// Saves all db instances to a file.
    pub fn save_all_db(&self) {
        let list = self.cache.read().unwrap();
        for (db_name, db) in list.iter() {
            let mut db_file = File::create(db_name.get_db_name()).expect(&format!(
                "Unable to create db file: {}",
                db_name.get_db_name()
            ));
            let db_lock = db.read().unwrap();
            let ser = serde_json::to_string(&db_lock.clone()).expect(&format!(
                "Unable to serialize db file: {}",
                db_name.get_db_name()
            ));
            let _ = db_file.write(ser.as_bytes()).expect(&format!(
                "Unable to write to db file: {}",
                db_name.get_db_name()
            ));
        }
    }

    /// Saves a specific db by name to file.
    /// Read locks the cache.
    pub fn save_specific_db(&self, db_name: &DBPacketInfo) {
        let list = self.cache.read().unwrap();
        match list.get(db_name) {
            Some(db_lock) => {
                let mut db_file = File::create(db_name.get_db_name()).expect(&format!(
                    "Unable to create db file: {}",
                    db_name.get_db_name()
                ));
                let db_clone = db_lock.read().unwrap().clone();
                let ser = serde_json::to_string(&db_clone).unwrap();
                let _ = db_file.write(ser.as_bytes()).expect(&format!(
                    "Unable to write to db file: {}",
                    db_name.get_db_name()
                ));
            }
            None => {
                panic!(
                    "Unable to save db: {}, db not found in list?",
                    db_name.get_db_name()
                );
            }
        }
    }

    /// Saves all db names to a file.
    pub fn save_db_list(&self) {
        let mut db_list_file = File::create("db_list.ser").expect("Unable to save db_list.ser");
        let ser_data = serde_json::to_string(&self).expect("Unable to serialize self.");
        let _ = db_list_file
            .write(ser_data.as_bytes())
            .expect("Unable to write bytes to db_list.ser");
    }

    /// Loads all db names from the db list file.
    pub fn load_db_list() -> Self {
        match File::open("db_list.ser") {
            Ok(mut f) => {
                // file found, load from file data
                let mut ser = String::new();
                f.read_to_string(&mut ser)
                    .expect("Unable to read db_list.ser to string");
                let db_list: Self =
                    serde_json::from_str(&ser).expect("Unable to deserialize db_list.ser");
                db_list
            }
            Err(_) => {
                // no file found, load default
                Self {
                    list: RwLock::new(vec![]),
                    cache: RwLock::new(HashMap::new()),
                    super_admin_hash_list: RwLock::new(vec![]),
                }
            }
        }
    }

    fn db_name_exists(&self, db_name: &str) -> bool {
        self.list
            .read()
            .unwrap()
            .contains(&DBPacketInfo::new(db_name))
    }

    /// Creates a DB given a name, the packet is not needed, only the name.
    pub fn create_db(&self, db_name: &str, db_settings: DBSettings, x: &String) -> DBPacketResponse<String> {

        if !self.is_super_admin(client_key) {
            // to create a db you must be a super admin
            return Error(InvalidPermissions);
        }

        if self.db_name_exists(db_name) {
            return Error(DBPacketResponseError::DBAlreadyExists);
        }

        let mut list_write_lock = self.list.write().unwrap();

        return match File::open(db_name) {
            Ok(_) => {
                // db file was found and should not have been, because this db already exists

                Error(DBPacketResponseError::DBAlreadyExists)
            }
            Err(_) => {
                // db file was not found
                match File::create(db_name) {
                    Ok(mut file) => {
                        let mut cache_write_lock = self.cache.write().unwrap();
                        let db_packet_info = DBPacketInfo::new(db_name);
                        let db = DB {
                            db_content: DBContent::default(),
                            last_access_time: SystemTime::now(),
                            db_settings,
                        };
                        let ser = serde_json::to_string(&db.db_content).unwrap();
                        let _ = file
                            .write(ser.as_ref())
                            .expect(&format!("Unable to write db to file. {}", db_name));
                        cache_write_lock.insert(db_packet_info.clone(), RwLock::from(db));
                        list_write_lock.push(db_packet_info);
                        SuccessNoData
                    }
                    Err(_) => {
                        // db file was unable to be created
                        Error(DBPacketResponseError::DBFileSystemError)
                    }
                }
            }
        };
    }

    /// Handles deleting a db, given a name for the db. Removes the database given a name, and deletes the corresponding file.
    /// If the file is successfully removed, the db is also removed from the cache, and list.
    pub fn delete_db(&self, db_name: &str, client_key: &String) -> DBPacketResponse<String> {

        if !self.is_super_admin(client_key) {
            // to delete a db, you must be a super admin no matter what.
            return Error(InvalidPermissions);
        }

        if !self.db_name_exists(db_name) {
            return Error(DBNotFound);
        }

        let mut list_lock = self.list.write().unwrap();

        let mut cache_lock = self.cache.write().unwrap();

        match fs::remove_file(db_name) {
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
                    return Error(DBPacketResponseError::DBFileSystemError);
                }

                SuccessNoData
            }
            Err(_) => Error(DBPacketResponseError::DBFileSystemError),
        }
    }

    /// Reads a database given a packet, returns the value if it was found.
    pub fn read_db(
        &self,
        p_info: &DBPacketInfo,
        p_location: &DBLocation,
    ) -> DBPacketResponse<String> {

        // TODO: use client key for read_db()

        let list_lock = self.list.read().unwrap();

        if let Some(db) = self.cache.read().unwrap().get(p_info) {
            // cache was hit
            db.write().unwrap().last_access_time = SystemTime::now();

            let db_lock = db.read().unwrap();

            let db_read = db_lock.db_content.read_from_db(p_location.as_key());

            return match db_read {
                None => Error(DBPacketResponseError::ValueNotFound),
                Some(value) => SuccessReply(value.to_string()),
            };
        }

        if list_lock.contains(p_info) {
            // cache was missed but the db exists on the file system

            let mut db_file = match File::open(p_info.get_db_name()) {
                Ok(f) => f,
                Err(_) => {
                    // early return db file system error when no file was able to be opened, should never happen due to the db file being in a list of known working db files.
                    return Error(DBPacketResponseError::DBFileSystemError);
                }
            };

            let mut db_content_string = String::new();
            db_file
                .read_to_string(&mut db_content_string)
                .expect("TODO: panic message");
            let mut db: DB = serde_json::from_str(&db_content_string).unwrap();

            db.last_access_time = SystemTime::now();

            let return_value = db
                .db_content
                .read_from_db(p_location.as_key())
                .expect("RETURN VALUE DID NOT EXIST")
                .clone();

            self.cache
                .write()
                .unwrap()
                .insert(p_info.clone(), RwLock::from(db));

            SuccessReply(return_value)
        } else {
            // cache was neither hit, nor did the db exist on the file system
            Error(DBNotFound)
        }
    }

    /// Writes to a db given a DBPacket
    pub fn write_db(
        &self,
        db_info: &DBPacketInfo,
        db_location: &DBLocation,
        db_data: DBData,
        client_key: &String,
    ) -> DBPacketResponse<String> {
        let list_lock = self.list.read().unwrap();

        {
            // scope the cache lock so it goes out of scope faster, allowing us to get a write lock later.
            let cache_lock = self.cache.read().unwrap();

            if let Some(db) = cache_lock.get(db_info) {
                // cache is hit, db is currently loaded

                let mut db_lock = db.write().unwrap();

                return if db_lock.has_write_permissions(client_key) {
                    db_lock.last_access_time = SystemTime::now();
                    match db_lock.db_content.content.insert(
                        db_location.as_key().to_string(),
                        db_data.get_data().to_string(),
                    ) {
                        None => {
                            // if the db insertion had no previous value, simply return an empty string, this could be updated later possibly.
                            SuccessNoData
                        }
                        Some(updated_value) => {
                            // if the db insertion had a previous value, return it.
                            SuccessReply(updated_value)
                        }
                    }
                } else {
                    Error(InvalidPermissions)
                }
            }
        }

        if list_lock.contains(db_info) {
            // cache was missed, but the requested database did in fact exist

            let mut cache_lock = self.cache.write().unwrap();

            let mut db_file = File::open(db_info.get_db_name()).unwrap();
            let mut db_content_string = String::new();
            db_file
                .read_to_string(&mut db_content_string)
                .expect("TODO: panic message");

            let mut db: DB = serde_json::from_str(&db_content_string).unwrap();

            db.last_access_time = SystemTime::now();

            if db.has_write_permissions(client_key) {
                let returned_value = db.db_content.content.insert(
                    db_location.as_key().to_string(),
                    db_data.get_data().to_string(),
                );

                cache_lock.insert(db_info.clone(), RwLock::from(db));

                match returned_value {
                    None => SuccessNoData,
                    Some(updated_value) => SuccessReply(updated_value),
                }
            } else {
                cache_lock.insert(db_info.clone(), RwLock::from(db));
                Error(InvalidPermissions)
            }


        } else {
            Error(DBNotFound)
        }
    }

    /// Returns the db list in a serialized form of Vec<DBPacketInfo>
    pub fn list_db(&self) -> DBPacketResponse<String> {
        let list = self.list.read().unwrap();
        match serde_json::to_string(&list.clone()) {
            Ok(thing) => SuccessReply(thing),
            Err(_) => Error(SerializationError),
        }
    }

    /// Returns the db contents in a serialized form of HashMap<String, String>
    pub fn list_db_contents(&self, db_info: &DBPacketInfo, client_key: &String) -> DBPacketResponse<String> {
        if !self.db_name_exists(db_info.get_db_name()) {
            return Error(DBNotFound);
        }

        let list_lock = self.list.read().unwrap();

        {
            // scope the cache lock so it goes out of scope faster, allowing us to get a write lock later.
            let cache_lock = self.cache.read().unwrap();

            if let Some(db) = cache_lock.get(db_info) {
                // cache is hit, db is currently loaded

                let mut db_lock = db.write().unwrap();

                return if db_lock.has_list_permissions(client_key) {
                    db_lock.last_access_time = SystemTime::now();

                    match serde_json::to_string(&db_lock.db_content.content) {
                        Ok(thing) => SuccessReply(thing),
                        Err(_) => Error(SerializationError),
                    }
                } else {
                    Error(InvalidPermissions)
                }
            }
        }

        if list_lock.contains(db_info) {
            // cache was missed, but the requested database did in fact exist

            let mut cache_lock = self.cache.write().unwrap();

            let mut db_file = File::open(db_info.get_db_name()).unwrap();
            let mut db_content_string = String::new();
            db_file
                .read_to_string(&mut db_content_string)
                .expect("TODO: panic message");

            let mut db: DB = serde_json::from_str(&db_content_string).unwrap();

            if db.has_list_permissions(client_key) {
                db.last_access_time = SystemTime::now();

                let returned_value = &db.db_content.content;

                let output_response = match serde_json::to_string(returned_value) {
                    Ok(thing) => SuccessReply(thing),
                    Err(_) => Error(SerializationError),
                };
                cache_lock.insert(db_info.clone(), RwLock::from(db));

                output_response
            } else {
                db.last_access_time = SystemTime::now();

                cache_lock.insert(db_info.clone(), RwLock::from(db));

                Error(InvalidPermissions)
            }
        } else {
            Error(DBNotFound)
        }
    }
}
