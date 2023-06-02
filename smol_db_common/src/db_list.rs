//! Contains structs and implementations for managing the active list of databases, that are both in filesystem, and in cache.
//! Also handles what to do when packets are received that modify any database that does or does not exist.
use crate::db::DB;
use crate::db_content::DBContent;
use crate::db_data::DBData;
use crate::db_packets::db_location::DBLocation;
use crate::db_packets::db_packet_info::DBPacketInfo;
use crate::db_packets::db_packet_response::{DBPacketResponse, DBPacketResponseError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct DBList {
    /// Vector of DBPacketInfo's containing file names of the databases that are available to be read from.
    pub list: RwLock<Vec<DBPacketInfo>>,

    /// Hashmap that takes a DBPacketInfo and returns the database corresponding to the name in the given packet.
    #[serde(skip)]
    pub cache: RwLock<HashMap<DBPacketInfo, Arc<RwLock<DB>>>>,
}

impl DBList {
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
    pub fn create_db(&self, db_name: &str) -> DBPacketResponse<String> {
        if self.db_name_exists(db_name) {
            return DBPacketResponse::Error(DBPacketResponseError::DBAlreadyExists);
        }

        let mut list_write_lock = self.list.write().unwrap();

        return match File::open(db_name) {
            Ok(_) => {
                // db file was found and should not have been, because this db already exists

                DBPacketResponse::Error(DBPacketResponseError::DBAlreadyExists)
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
                        };
                        let ser = serde_json::to_string(&db.db_content).unwrap();
                        let _ = file
                            .write(ser.as_ref())
                            .expect(&format!("Unable to write db to file. {}", db_name));
                        cache_write_lock
                            .insert(db_packet_info.clone(), Arc::from(RwLock::from(db)));
                        list_write_lock.push(db_packet_info);
                        DBPacketResponse::SuccessNoData
                    }
                    Err(_) => {
                        // db file was unable to be created
                        DBPacketResponse::Error(DBPacketResponseError::DBFileSystemError)
                    }
                }
            }
        };
    }

    /// Handles deleting a db, given a name for the db. Removes the database given a name, and deletes the corresponding file.
    /// If the file is successfully removed, the db is also removed from the cache, and list.
    pub fn delete_db(&self, db_name: &str) -> DBPacketResponse<String> {
        if !self.db_name_exists(db_name) {
            return DBPacketResponse::Error(DBPacketResponseError::DBNotFound);
        }

        let mut list_lock = self.list.write().unwrap();

        let mut cache_lock = self.cache.write().unwrap();

        match fs::remove_file(db_name) {
            Ok(_) => {
                let db_packet_info = DBPacketInfo::new(db_name);
                cache_lock.remove(&db_packet_info);
                let index_res = list_lock.binary_search(&db_packet_info);
                if let Ok(index) = index_res {
                    list_lock.remove(index);
                }
                DBPacketResponse::SuccessNoData
            }
            Err(_) => DBPacketResponse::Error(DBPacketResponseError::DBFileSystemError),
        }
    }

    /// Reads a database given a packet, returns the value if it was found.
    pub fn read_db(
        &self,
        p_info: DBPacketInfo,
        p_location: DBLocation,
    ) -> DBPacketResponse<String> {
        let list_lock = self.list.read().unwrap();

        if let Some(db) = self.cache.read().unwrap().get(&p_info) {
            // cache was hit
            db.write().unwrap().last_access_time = SystemTime::now();

            return DBPacketResponse::SuccessReply(
                db.read()
                    .unwrap()
                    .db_content
                    .read_from_db(p_location.as_key())
                    .unwrap()
                    .to_string(),
            );
        }

        if list_lock.contains(&p_info) {
            // cache was missed but the db exists on the file system

            let mut db_file = match File::open(p_info.get_db_name()) {
                Ok(f) => f,
                Err(_) => {
                    // early return db file system error when no file was able to be opened, should never happen due to the db file being in a list of known working db files.
                    return DBPacketResponse::Error(DBPacketResponseError::DBFileSystemError);
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
                .insert(p_info, Arc::from(RwLock::from(db)));

            DBPacketResponse::SuccessReply(return_value)
        } else {
            // cache was neither hit, nor did the db exist on the file system
            DBPacketResponse::Error(DBPacketResponseError::DBNotFound)
        }
    }

    /// Writes to a db given a DBPacket
    pub fn write_db(
        &self,
        db_info: &DBPacketInfo,
        db_location: &DBLocation,
        db_data: DBData,
    ) -> DBPacketResponse<String> {
        let list_lock = self.list.read().unwrap();
        let cache_lock = self.cache.read().unwrap();

        if let Some(db) = cache_lock.get(db_info) {
            // cache is hit, db is currently loaded

            let mut db_lock = db.write().unwrap();

            db_lock.last_access_time = SystemTime::now();
            return match db_lock.db_content.content.insert(
                db_location.as_key().to_string(),
                db_data.get_data().to_string(),
            ) {
                None => {
                    // if the db insertion had no previous value, simply return an empty string, this could be updated later possibly.
                    DBPacketResponse::SuccessNoData
                }
                Some(updated_value) => {
                    // if the db insertion had a previous value, return it.
                    DBPacketResponse::SuccessReply(updated_value)
                }
            };
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

            let returned_value = db.db_content.content.insert(
                db_location.as_key().to_string(),
                db_data.get_data().to_string(),
            );

            cache_lock.insert(db_info.clone(), Arc::from(RwLock::from(db)));

            match returned_value {
                None => DBPacketResponse::SuccessNoData,
                Some(updated_value) => DBPacketResponse::SuccessReply(updated_value),
            }
        } else {
            DBPacketResponse::Error(DBPacketResponseError::DBNotFound)
        }
    }
}
