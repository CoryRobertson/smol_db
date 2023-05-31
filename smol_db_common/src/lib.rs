//! Common library between the client and server for smol_db
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// A struct that describes the name of a database to be searched through.
pub struct DBPacketInfo {
    dbname: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct that describes a key to search with through the database.
pub struct DBLocation {
    location: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct that contains the data that is to be put into a database.
pub struct DBData {
    data: String,
}

impl DBData {
    /// Function to create a new DBData struct for a DBPacket::Write packet.
    pub fn new(data: String) -> Self {
        // TODO: eventually revise this with some amount of error checking
        Self { data }
    }

    /// Getter function for the data inside the DBData struct.
    pub fn get_data(&self) -> &str {
        &self.data
    }
}

impl DBLocation {
    /// Function to create a new DBLocation struct from a given location.
    pub fn new(location: &str) -> Self {
        Self {
            location: location.to_string(),
        }
    }

    /// Function to retrieve the location as a key from the struct.
    pub fn as_key(&self) -> &str {
        &self.location
    }
}

impl DBPacketInfo {
    /// Function to create a new DBPacketInfo struct with the given name
    pub fn new(dbname: &str) -> Self {
        Self {
            dbname: dbname.to_string(),
        }
    }

    /// Function to retrieve the name from the DBPacketInfo struct.
    pub fn get_db_name(&self) -> &str {
        &self.dbname
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A packet denoting the operation from client->server that the client wishes to do.
pub enum DBPacket {
    /// Read(db to operate on, key to read the db using)
    Read(DBPacketInfo, DBLocation),
    /// Write(db to operate on, key to write to the db using, data to write to the key location)
    Write(DBPacketInfo, DBLocation, DBData),
    /// CreateDB(db to create)
    CreateDB(DBPacketInfo),
    /// DeleteDB(db to delete)
    DeleteDB(DBPacketInfo),
}

#[derive(Serialize, Deserialize, Debug)]
/// This enum represents the various types of responses that accessing the database can be.
pub enum DBPacketResponse<T> {
    /// DBPacketResponse is a response type for a DBPacket request
    SuccessNoData,
    /// SuccessNoData represents when the operation was successful, but no response data was necessary to be replied back.
    SuccessReply(T),
    /// Error represents any issue when interacting with a database in general, it contains a further description of the error inside.
    Error(DBPacketResponseError),
}

#[derive(Serialize, Deserialize, Debug)]
/// This enum represents the various types of errors that can occur when an error is returned in a db packet response
pub enum DBPacketResponseError {
    /// BadPacket represents a packet that was improperly handled, these should be reported immediately and should never happen under proper circumstances.
    BadPacket,
    /// DBNotFound represents a request to a database that does not exist.
    DBNotFound,
    /// DBFileSystemError represents an issue loading or reading the file that contains a given database, not necessarily it not existing.
    DBFileSystemError,
    /// ValueNotFound represents when a given value in a database does not exist.
    ValueNotFound,
}

impl DBPacket {
    /// Creates a new Read DBPacket from a name of a database and location string to read from.
    pub fn new_read(dbname: &str, location: &str) -> DBPacket {
        DBPacket::Read(DBPacketInfo::new(dbname), DBLocation::new(location))
    }

    /// Creates a new Write DBPacket from a name of a database and location string to write to.
    pub fn new_write(dbname: &str, location: &str, data: &str) -> DBPacket {
        DBPacket::Write(
            DBPacketInfo::new(dbname),
            DBLocation::new(location),
            DBData::new(data.to_string()),
        )
    }

    /// Creates a new CreateDB DBPacket from a name of a database.
    pub fn new_create_db(dbname: &str) -> DBPacket {
        DBPacket::CreateDB(DBPacketInfo::new(dbname))
    }

    /// Creates a new DeleteDB DBPacket from a name of a database.
    pub fn new_delete_db(dbname: &str) -> DBPacket {
        DBPacket::DeleteDB(DBPacketInfo::new(dbname))
    }

    /// Serializes a DBPacket into a string to be sent over the internet.
    pub fn serialize_packet(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Struct denoting the content itself of a database. At the moment, is simply a hashmap.
pub struct DBContent {
    pub content: HashMap<String, String>,
}

impl DBContent {
    /// Reads serialized version of a DBContent struct from a string (read from a file most likely) into a DBContent struct itself.
    pub fn read_ser_data(data: String) -> serde_json::Result<Self> {
        serde_json::from_str(&data)
    }

    /// Reads from the db using the key, returning an optional of either the retrieved content, or nothing.
    pub fn read_from_db(&self, key: &str) -> Option<&String> {
        self.content.get(key)
    }
}

#[allow(clippy::derivable_impls)] // This lint is allowed so we can later make default not simply have the default impl
impl Default for DBContent {
    /// Returns a default empty HashMap.
    fn default() -> Self {
        Self {
            content: HashMap::default(),
        }
    }
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A struct that represents a specific database, with content, and a recent access time.
/// This struct is meant to be called into existence when ever a database is un-cached, and needs to be cached.
pub struct DB {
    pub db_content: DBContent,
    last_access_time: SystemTime,
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
