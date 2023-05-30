use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};


#[derive(Serialize,Deserialize,Debug,Clone, Hash, PartialEq, Eq,PartialOrd,Ord)]
/// A struct that describes the name of a database to be searched through.
pub struct DBPacketInfo {
    dbname: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
/// A struct that describes a key to search with through the database.
pub struct DBLocation {
    location: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
/// A struct that contains the data that is to be put into a database.
pub struct DBData {
    data: String
}

impl DBData {
    /// Function to create a new DBData struct for a DBPacket::Write packet.
    pub fn new(data: String) -> Self {
        // TODO: eventually revise this with some amount of error checking
        Self {
            data
        }
    }

    /// Getter function for the data inside the DBData struct.
    pub fn get_data(&self) -> &str {
        &self.data
    }
}

impl DBLocation {
    /// Function to create a new DBLocation struct from a given location.
    pub fn new(location: &str) -> Self {
        Self{ location: location.to_string() }
    }

    /// Function to retrieve the location as a key from the struct.
    pub fn as_key(&self) -> &str {
        &self.location
    }
}

impl DBPacketInfo {
    /// Function to create a new DBPacketInfo struct with the given name
    pub fn new(dbname: &str) -> Self {
        Self{ dbname: dbname.to_string() }
    }

    /// Function to retrieve the name from the DBPacketInfo struct.
    pub fn get_db_name(&self) -> &str {
        &self.dbname
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
/// A packet denoting the operation from client->server that the client wishes to do.
///
/// Read(db to operate on, key to read the db using)
/// Write(db to operate on, key to write to the db using)
/// CreateDB(db to create)
/// DeleteDB(db to delete)
pub enum DBPacket {
    Read(DBPacketInfo, DBLocation),
    Write(DBPacketInfo, DBLocation, DBData),
    CreateDB(DBPacketInfo),
    DeleteDB(DBPacketInfo),
}

// TODO: write a DBPacketResponse enum that represents the types of responses that the database server can give
//  Examples include:
//  SUCCESS, meaning the response sent was successful, but needed no further information to be returned.
//  REPLY(StructContainingTheData(String?))
//  Error(SomeEnumErrorType)
//  For StructContainingTheData:
//  The struct could be a serializable struct that can be deserialized?
//  Most likely just a simple String so we can retain Serialization.
//  But could be an enum that denotes type so we dont have to guess what the type of the info is?
//  For SomeEnumErrorType:
//  Likely just a few different error types including DBNotFound, DataNotFound, anything else?

impl DBPacket {
    /// Creates a new Read DBPacket from a name of a database and location string to read from.
    pub fn new_read(dbname: &str, location: &str) -> DBPacket {
        DBPacket::Read(DBPacketInfo::new(dbname),DBLocation::new(location))
    }

    /// Creates a new Write DBPacket from a name of a database and location string to write to.
    pub fn new_write(dbname: &str, location: &str, data: &str) -> DBPacket {
        DBPacket::Write(DBPacketInfo::new(dbname),DBLocation::new(location), DBData::new(data.to_string()))
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

#[derive(Serialize,Deserialize,Debug,Clone)]
/// Struct denoting the content itself of a database. At the moment, is simply a hashmap.
pub struct DBContent {
    pub content: HashMap<String,String>,
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
impl Default for DBContent{
    /// Returns a default empty HashMap.
    fn default() -> Self {
        Self{ content: HashMap::default() }
    }
}

// TODO: move these structs and impl blocks to lib.rs

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DBList {
    //TODO: store the cache and list in an RWLock, and eventually store each DB in the cache in an RWLock so individual databases can be read from and written to concurrently.
    //  These should allow us to read/write from each individual database concurrently.
    //  Something like RWLock<HashMap<DBPacketInfo,RWLock<DB>>>
    //  And RWLock<Vec<DBPacketInfo>>
    pub list: Vec<DBPacketInfo>, // vector of strings containing file names of the databases.
    pub cache: HashMap<DBPacketInfo,DB>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DB {
    pub db_content: DBContent,
    last_access_time: SystemTime,
}

impl DBList {
    pub fn create_db(&mut self, db_name: &str) -> std::io::Result<File> {
        let mut res = File::create(db_name);

        if let Ok(file) = &mut res {
            let db_packet_info = DBPacketInfo::new(db_name);
            let db = DB { db_content: DBContent::default(), last_access_time: SystemTime::now() };
            let ser = serde_json::to_string(&db.db_content).unwrap();
            let _ = file.write(ser.as_ref()).expect("TODO: panic message");
            self.cache.insert(db_packet_info.clone(), db);
            self.list.push(db_packet_info);

        }

        res
    }
    pub fn delete_db(&mut self, db_name: &str) -> std::io::Result<()> {
        let res = fs::remove_file(db_name);

        if res.is_ok() {
            let db_packet_info = DBPacketInfo::new(db_name);
            self.cache.remove(&db_packet_info);
            let index_res = self.list.binary_search(&db_packet_info);
            if let Ok(index) = index_res {
                self.list.remove(index);
            }
        }

        res
    }
    // TODO: modify these result return types to use TODO referencing DBPacketResponse
    pub fn read_db(&mut self, read_pack: &DBPacket) -> Result<String,()> {
        return match read_pack {
            DBPacket::Read(p_info, p_location) => {
                if let Some(db) = self.cache.get_mut(p_info) {
                    // cache was hit
                    db.last_access_time = SystemTime::now();

                    Ok(db.db_content.read_from_db(p_location.as_key()).unwrap().to_string())
                } else if self.list.contains(p_info) {
                    // cache was missed but the db exists on the file system

                    let mut db_file = File::open(p_info.get_db_name()).unwrap();
                    let mut db_content_string = String::new();
                    db_file.read_to_string(&mut db_content_string).expect("TODO: panic message");
                    let db_content: DBContent = DBContent::read_ser_data(db_content_string).unwrap();

                    let return_value = db_content.read_from_db(p_location.as_key()).expect("RETURN VALUE DID NOT EXIST").clone();

                    let db = DB { db_content, last_access_time: SystemTime::now() };
                    self.cache.insert(p_info.clone(), db);


                    Ok(return_value)
                } else {
                    // cache was neither hit, nor did the db exist on the file system
                    Err(())
                }
            }
            DBPacket::Write(_, _, _) => { Err(()) }
            DBPacket::CreateDB(_) => { Err(()) }
            DBPacket::DeleteDB(_) => { Err(()) }
        };
    }

    pub fn write_db(&mut self, write_pack: &DBPacket) -> Result<String,()> {
        return match write_pack {
            DBPacket::Read(_, _) => { Err(()) }
            DBPacket::Write(db_info, db_location, db_data) => {
                if let Some(db) = self.cache.get_mut(db_info) {
                    // cache is hit, db is currently loaded
                    db.last_access_time = SystemTime::now();
                    return match db.db_content.content.insert(db_location.as_key().to_string(),db_data.get_data().to_string()) {
                        None => {
                            // if the db insertion had no previous value, simply return an empty string, this could be updated later possibly.
                            Ok("".to_string())
                        }
                        Some(updated_value) => {
                            // if the db insertion had a previous value, return it.
                            Ok(updated_value)
                        }
                    }
                } else if self.list.contains(db_info) {
                    // cache was missed, but the requested database did in fact exist
                    let mut db_file = File::open(db_info.get_db_name()).unwrap();
                    let mut db_content_string = String::new();
                    db_file.read_to_string(&mut db_content_string).expect("TODO: panic message");
                    let db_content: DBContent = DBContent::read_ser_data(db_content_string).unwrap();

                    let mut db = DB { db_content, last_access_time: SystemTime::now() };
                    let returned_value = match db.db_content.content.insert(db_location.as_key().to_string(),db_data.get_data().to_string()) {
                        None => { "".to_string() }
                        Some(updated_value) => {
                            updated_value
                        }
                    };
                    self.cache.insert(db_info.clone(), db);

                    Ok(returned_value)
                } else {
                    Err(())
                }
            }
            DBPacket::CreateDB(_) => { Err(()) }
            DBPacket::DeleteDB(_) => { Err(()) }
        }
    }
}
