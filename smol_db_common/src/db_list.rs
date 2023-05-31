//! Contains structs and implementations for managing the active list of databases, that are both in filesystem, and in cache.
//! Also handles what to do when packets are received that modify any database that does or does not exist.
use crate::db::DB;
use crate::db_content::DBContent;
use crate::db_packets::db_packet::DBPacket;
use crate::db_packets::db_packet_info::DBPacketInfo;
use crate::db_packets::db_packet_response::{DBPacketResponse, DBPacketResponseError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBList {
    //TODO: store the cache and list in an RWLock, and eventually store each DB in the cache in an RWLock so individual databases can be read from and written to concurrently.
    //  These should allow us to read/write from each individual database concurrently.
    //  Something like RWLock<HashMap<DBPacketInfo,RWLock<DB>>>
    //  And RWLock<Vec<DBPacketInfo>>
    /// Vector of DBPacketInfo's containing file names of the databases that are available to be read from.
    pub list: Vec<DBPacketInfo>,
    /// Hashmap that takes a DBPacketInfo and returns the database corresponding to the name in the given packet.
    pub cache: HashMap<DBPacketInfo, DB>,
}

impl DBList {
    /// Creates a DB given a name, the packet is not needed, only the name.
    pub fn create_db(&mut self, db_name: &str) -> DBPacketResponse<()> {
        return match File::create(db_name) {
            Ok(mut file) => {
                let db_packet_info = DBPacketInfo::new(db_name);
                let db = DB {
                    db_content: DBContent::default(),
                    last_access_time: SystemTime::now(),
                };
                let ser = serde_json::to_string(&db.db_content).unwrap();
                let _ = file.write(ser.as_ref()).expect("TODO: panic message");
                self.cache.insert(db_packet_info.clone(), db);
                self.list.push(db_packet_info);
                DBPacketResponse::SuccessNoData
            }
            Err(_) => DBPacketResponse::Error(DBPacketResponseError::DBFileSystemError),
        };
    }

    /// Handles deleting a db, given a name for the db. Removes the database given a name, and deletes the corresponding file.
    /// If the file is successfully removed, the db is also removed from the cache, and list.
    pub fn delete_db(&mut self, db_name: &str) -> DBPacketResponse<()> {
        match fs::remove_file(db_name) {
            Ok(_) => {
                let db_packet_info = DBPacketInfo::new(db_name);
                self.cache.remove(&db_packet_info);
                let index_res = self.list.binary_search(&db_packet_info);
                if let Ok(index) = index_res {
                    self.list.remove(index);
                }
                DBPacketResponse::SuccessNoData
            }
            Err(_) => DBPacketResponse::Error(DBPacketResponseError::DBFileSystemError),
        }
    }

    /// Reads a database given a packet, returns the value if it was found.
    pub fn read_db(&mut self, read_pack: &DBPacket) -> DBPacketResponse<String> {
        return match read_pack {
            DBPacket::Read(p_info, p_location) => {
                if let Some(db) = self.cache.get_mut(p_info) {
                    // cache was hit
                    db.last_access_time = SystemTime::now();

                    DBPacketResponse::SuccessReply(
                        db.db_content
                            .read_from_db(p_location.as_key())
                            .unwrap()
                            .to_string(),
                    )
                } else if self.list.contains(p_info) {
                    // cache was missed but the db exists on the file system

                    let mut db_file = match File::open(p_info.get_db_name()) {
                        Ok(f) => f,
                        Err(_) => {
                            // early return db file system error when no file was able to be opened, should never happen due to the db file being in a list of known working db files.
                            return DBPacketResponse::Error(
                                DBPacketResponseError::DBFileSystemError,
                            );
                        }
                    };
                    let mut db_content_string = String::new();
                    db_file
                        .read_to_string(&mut db_content_string)
                        .expect("TODO: panic message");
                    let db_content: DBContent =
                        DBContent::read_ser_data(db_content_string).unwrap();

                    let return_value = db_content
                        .read_from_db(p_location.as_key())
                        .expect("RETURN VALUE DID NOT EXIST")
                        .clone();

                    let db = DB {
                        db_content,
                        last_access_time: SystemTime::now(),
                    };
                    self.cache.insert(p_info.clone(), db);

                    DBPacketResponse::SuccessReply(return_value)
                } else {
                    // cache was neither hit, nor did the db exist on the file system
                    DBPacketResponse::Error(DBPacketResponseError::DBNotFound)
                }
            }
            DBPacket::Write(_, _, _) => DBPacketResponse::Error(DBPacketResponseError::BadPacket),
            DBPacket::CreateDB(_) => DBPacketResponse::Error(DBPacketResponseError::BadPacket),
            DBPacket::DeleteDB(_) => DBPacketResponse::Error(DBPacketResponseError::BadPacket),
        };
    }

    /// Writes to a db given a DBPacket
    pub fn write_db(&mut self, write_pack: &DBPacket) -> DBPacketResponse<String> {
        return match write_pack {
            DBPacket::Write(db_info, db_location, db_data) => {
                if let Some(db) = self.cache.get_mut(db_info) {
                    // cache is hit, db is currently loaded
                    db.last_access_time = SystemTime::now();
                    return match db.db_content.content.insert(
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
                } else if self.list.contains(db_info) {
                    // cache was missed, but the requested database did in fact exist
                    let mut db_file = File::open(db_info.get_db_name()).unwrap();
                    let mut db_content_string = String::new();
                    db_file
                        .read_to_string(&mut db_content_string)
                        .expect("TODO: panic message");
                    let db_content: DBContent =
                        DBContent::read_ser_data(db_content_string).unwrap();

                    let mut db = DB {
                        db_content,
                        last_access_time: SystemTime::now(),
                    };
                    let returned_value = db.db_content.content.insert(
                        db_location.as_key().to_string(),
                        db_data.get_data().to_string(),
                    );
                    self.cache.insert(db_info.clone(), db);

                    return match returned_value {
                        None => DBPacketResponse::SuccessNoData,
                        Some(updated_value) => DBPacketResponse::SuccessReply(updated_value),
                    };
                } else {
                    DBPacketResponse::Error(DBPacketResponseError::DBNotFound)
                }
            }
            // Error on any incorrect packet types.
            DBPacket::Read(_, _) => DBPacketResponse::Error(DBPacketResponseError::BadPacket),
            DBPacket::CreateDB(_) => DBPacketResponse::Error(DBPacketResponseError::BadPacket),
            DBPacket::DeleteDB(_) => DBPacketResponse::Error(DBPacketResponseError::BadPacket),
        };
    }
}
